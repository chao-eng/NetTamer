//! WinDivert capture engine: receive -> classify -> throttle -> re-inject/drop.
//!
//! The engine owns a WinDivert handle and, on each captured packet, resolves the
//! owning PID via the shared [`PortPidMap`], checks the active
//! [`ThrottleTable`] for that PID, and either re-injects the packet (admitted)
//! or drops it (throttled). Download (inbound) buffering + controlled re-send is
//! stubbed with a TODO — for v2.0 the simplest correct behaviour is to drop
//! excess inbound packets and let TCP congestion control recover.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use std::thread;

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
    throttle: Arc<ThrottleTable>,
    port_map: Arc<PortPidMap>,
    resolver: Arc<Resolver>,
}

impl WinDivertEngine {
    /// Open a WinDivert handle with the given filter expression (e.g. "tcp or udp").
    pub fn open(
        filter: &str,
        throttle: Arc<ThrottleTable>,
        port_map: Arc<PortPidMap>,
        resolver: Arc<Resolver>,
    ) -> Result<Self, crate::models::Error> {
        let c_filter =
            std::ffi::CString::new(filter).map_err(|e| crate::models::Error(e.to_string()))?;
        // WinDivert 1.4 uses an ANSI filter string.
        let handle = unsafe {
            WinDivertOpen(
                c_filter.as_ptr() as *const u8,
                WINDIVERT_LAYER_NETWORK,
                0,
                0,
            )
        };
        if handle == INVALID_HANDLE {
            return Err(crate::models::Error::new(
                "WinDivertOpen failed: is WinDivert.dll/.sys present and running as administrator?",
            ));
        }
        Ok(Self {
            handle: Arc::new(Mutex::new(Some(RawHandle(handle)))),
            throttle,
            port_map,
            resolver,
        })
    }

    /// Spawn the capture loop on a background thread. The thread exits when the
    /// handle is closed via [`WinDivertEngine::stop`].
    pub fn run(self: &Arc<Self>) {
        let handle = self.handle.clone();
        let throttle = self.throttle.clone();
        let port_map = self.port_map.clone();
        let resolver = self.resolver.clone();

        thread::spawn(move || {
            let raw = match *handle.lock().unwrap() {
                Some(h) => h.0,
                None => return,
            };
            let mut buf = vec![0u8; 65536];
            let mut addr = WINDIVERT_ADDRESS {
                if_idx: 0,
                sub_if_idx: 0,
                direction: 0,
            };

            loop {
                // Bail out if the handle was closed by `stop()`.
                if handle.lock().unwrap().is_none() {
                    break;
                }
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
                    // Receive failed (handle closed or driver error) -> stop.
                    break;
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
                if let Some(pid) = Self::classify(&buf[..len], is_outbound, &port_map) {
                    let name = resolver.resolve(pid).name.clone();
                    throttle.note_process(&name, pid);
                    if let Some(policy) = throttle.policy_for_name(&name) {
                        if direction.matches_policy(&policy) && policy.rate_limit_bps > 0 {
                            allowed = throttle.admit(pid, &policy, len);
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
                    // Throttled: drop the packet.
                }
            }
        });
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

    /// Change the capture filter. WinDivert has no live re-filter API, so this
    /// is a placeholder for a close/reopen cycle (TODO).
    #[allow(dead_code)]
    pub fn set_filter(&self, _filter: &str) -> Result<(), crate::models::Error> {
        // TODO: implement close-and-reopen with the new filter expression.
        Ok(())
    }

    /// Close the WinDivert handle, stopping the capture loop.
    pub fn stop(&self) -> Result<(), crate::models::Error> {
        let mut guard = self.handle.lock().unwrap();
        if let Some(h) = guard.take() {
            unsafe {
                WinDivertClose(h.0);
            }
        }
        Ok(())
    }
}
