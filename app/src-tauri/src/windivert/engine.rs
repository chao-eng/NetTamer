//! WinDivert capture engine: receive -> classify -> throttle -> re-inject/drop.
//!
//! Only intercepts packets matching the active ports of specified target processes (Plan A).
//! Traffic from all other applications is completely untouched and bypasses NetTamer.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::raw::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use crate::models::Direction;
use crate::process::{PortPidMap, Resolver};
use crate::throttle::ThrottleTable;

use crate::windivert::ffi::{
    WinDivertClose, WinDivertOpen, WinDivertRecv, WinDivertSend, INVALID_HANDLE,
    WINDIVERT_ADDRESS, WINDIVERT_LAYER_NETWORK,
};

#[derive(Clone, Copy)]
pub struct RawHandle(pub *mut c_void);
unsafe impl Send for RawHandle {}
unsafe impl Sync for RawHandle {}

pub struct WinDivertEngine {
    handle: Arc<Mutex<Option<RawHandle>>>,
    is_running: Arc<AtomicBool>,
    target_names: Arc<RwLock<Vec<String>>>,
    throttle: Arc<ThrottleTable>,
    port_map: Arc<PortPidMap>,
    resolver: Arc<Resolver>,
}

impl WinDivertEngine {
    /// Start the capture engine dedicated to the specified target processes.
    pub fn start(
        target_names: Vec<String>,
        throttle: Arc<ThrottleTable>,
        port_map: Arc<PortPidMap>,
        resolver: Arc<Resolver>,
    ) -> Result<Arc<Self>, crate::models::Error> {
        let is_running = Arc::new(AtomicBool::new(true));
        let handle = Arc::new(Mutex::new(None));
        let target_names = Arc::new(RwLock::new(target_names));

        let engine = Arc::new(Self {
            handle: handle.clone(),
            is_running: is_running.clone(),
            target_names: target_names.clone(),
            throttle,
            port_map,
            resolver,
        });

        let eng = engine.clone();

        // Background worker thread managing the capture loop and dynamic port changes
        thread::spawn(move || {
            let mut current_ports: Vec<u16> = Vec::new();
            let mut last_port_check = std::time::Instant::now() - Duration::from_secs(10);

            while eng.is_running.load(Ordering::Relaxed) {
                // 1. Periodically query active ports for target process names
                let names = eng.target_names.read().unwrap().clone();
                let fresh_ports = eng.port_map.get_ports_for_process_names(&names, &eng.resolver);

                // 2. If ports changed or handle not open yet, reopen handle with precise filter
                let handle_is_none = eng.handle.lock().unwrap().is_none();
                if fresh_ports != current_ports || handle_is_none {
                    // Close stale handle
                    if let Some(h) = eng.handle.lock().unwrap().take() {
                        unsafe {
                            WinDivertClose(h.0);
                        }
                    }

                    current_ports = fresh_ports.clone();
                    let filter = if current_ports.is_empty() {
                        "false".to_string()
                    } else {
                        let clauses: Vec<String> = current_ports
                            .iter()
                            .map(|p| {
                                format!(
                                    "tcp.SrcPort == {p} or tcp.DstPort == {p} or udp.SrcPort == {p} or udp.DstPort == {p}"
                                )
                            })
                            .collect();
                        clauses.join(" or ")
                    };

                    if let Ok(c_filter) = std::ffi::CString::new(filter) {
                        let raw = unsafe {
                            WinDivertOpen(
                                c_filter.as_ptr() as *const u8,
                                WINDIVERT_LAYER_NETWORK,
                                0,
                                0,
                            )
                        };
                        if raw != INVALID_HANDLE {
                            *eng.handle.lock().unwrap() = Some(RawHandle(raw));
                        }
                    }
                    last_port_check = std::time::Instant::now();
                }

                // 3. If target currently has no open ports (idle or not launched), sleep briefly and check again
                if current_ports.is_empty() {
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }

                let raw = match *eng.handle.lock().unwrap() {
                    Some(h) => h.0,
                    None => {
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                };

                let mut buf = vec![0u8; 65536];
                let mut addr = WINDIVERT_ADDRESS {
                    if_idx: 0,
                    sub_if_idx: 0,
                    direction: 0,
                };
                let mut recv_len: u32 = 0;

                let ok = unsafe {
                    WinDivertRecv(
                        raw,
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len() as u32,
                        &mut addr,
                        &mut recv_len,
                    )
                };

                if ok == 0 {
                    // Receive failed (handle closed due to port update or stop)
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                let len = recv_len as usize;
                if len == 0 {
                    continue;
                }

                let is_outbound = addr.direction == 0;
                let direction = if is_outbound {
                    Direction::Send
                } else {
                    Direction::Recv
                };

                let mut allowed = true;
                if let Some(pid) = Self::classify(&buf[..len], is_outbound, &eng.port_map) {
                    let name = eng.resolver.resolve(pid).name.clone();
                    eng.throttle.note_process(&name, pid);
                    if let Some(policy) = eng.throttle.policy_for_name(&name) {
                        if direction.matches_policy(&policy) && policy.rate_limit_bps > 0 {
                            allowed = eng.throttle.admit(pid, &policy, len);
                        }
                    }
                }

                if allowed {
                    let mut send_len: u32 = 0;
                    unsafe {
                        WinDivertSend(
                            raw,
                            buf.as_ptr() as *const c_void,
                            len as u32,
                            &addr,
                            &mut send_len,
                        );
                    }
                } else {
                    // Throttled: packet dropped.
                }

                // Periodically (every 1s) check if ports changed and unblock if needed
                if last_port_check.elapsed() > Duration::from_millis(1000) {
                    last_port_check = std::time::Instant::now();
                    let names = eng.target_names.read().unwrap().clone();
                    let check_ports = eng.port_map.get_ports_for_process_names(&names, &eng.resolver);
                    if check_ports != current_ports {
                        if let Some(h) = eng.handle.lock().unwrap().take() {
                            unsafe {
                                WinDivertClose(h.0);
                            }
                        }
                    }
                }
            }

            // Cleanup when stopped
            if let Some(h) = eng.handle.lock().unwrap().take() {
                unsafe {
                    WinDivertClose(h.0);
                }
            }
        });

        Ok(engine)
    }

    /// Update target process names dynamically
    pub fn update_targets(&self, targets: Vec<String>) {
        *self.target_names.write().unwrap() = targets;
        // Unblock current handle to trigger re-bind immediately
        if let Some(h) = self.handle.lock().unwrap().take() {
            unsafe {
                WinDivertClose(h.0);
            }
        }
    }

    /// Best-effort IPv4/TCP|UDP parser -> owning PID via the port map.
    fn classify(pkt: &[u8], is_outbound: bool, port_map: &PortPidMap) -> Option<u32> {
        if pkt.len() < 20 {
            return None;
        }
        let ip_version = pkt[0] >> 4;
        if ip_version != 4 {
            return None; // TODO: IPv6
        }
        let ihl = (pkt[0] & 0x0f) as usize * 4;
        if pkt.len() < ihl + 4 {
            return None;
        }
        let proto = pkt[9];
        if proto != 6 && proto != 17 {
            return None; // only TCP / UDP
        }
        let src_ip = u32::from_be_bytes([pkt[12], pkt[13], pkt[14], pkt[15]]);
        let dst_ip = u32::from_be_bytes([pkt[16], pkt[17], pkt[18], pkt[19]]);
        let src_port = u16::from_be_bytes([pkt[ihl], pkt[ihl + 1]]);
        let dst_port = u16::from_be_bytes([pkt[ihl + 2], pkt[ihl + 3]]);

        let saddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(src_ip)), src_port);
        let daddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(dst_ip)), dst_port);

        let (local_addr, local_port) = if is_outbound {
            (saddr, src_port)
        } else {
            (daddr, dst_port)
        };

        port_map
            .lookup(&local_addr)
            .or_else(|| port_map.lookup_port(local_port))
            .or_else(|| {
                // Secondary fallback: test both sides
                let remote_addr = if is_outbound { daddr } else { saddr };
                let remote_port = if is_outbound { dst_port } else { src_port };
                port_map
                    .lookup(&remote_addr)
                    .or_else(|| port_map.lookup_port(remote_port))
            })
    }

    /// Stop the WinDivert capture loop and close handle.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        let mut guard = self.handle.lock().unwrap();
        if let Some(h) = guard.take() {
            unsafe {
                WinDivertClose(h.0);
            }
        }
    }
}
