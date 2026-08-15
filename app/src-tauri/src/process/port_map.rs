//! Socket -> PID mapping used to attribute captured packets to a process.
//!
//! Refreshes from the OS TCP table (`GetExtendedTcpTable`) and UDP table
//! (`GetExtendedUdpTable`). Provides lookups by exact SocketAddr and by port.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::RwLock;
use std::time::Instant;

use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows_sys::Win32::Networking::WinSock::AF_INET;

pub struct PortPidMap {
    addr_map: RwLock<HashMap<SocketAddr, u32>>,
    port_map: RwLock<HashMap<u16, u32>>,
    last_refresh: RwLock<Instant>,
}

#[repr(C)]
struct MibTcpRowOwnerPid {
    dw_state: u32,
    dw_local_addr: u32,
    dw_local_port: u32,
    dw_remote_addr: u32,
    dw_remote_port: u32,
    dw_owning_pid: u32,
}

#[repr(C)]
struct MibUdpRowOwnerPid {
    dw_local_addr: u32,
    dw_local_port: u32,
    dw_owning_pid: u32,
}

impl PortPidMap {
    pub fn new() -> Self {
        let instance = PortPidMap {
            addr_map: RwLock::new(HashMap::new()),
            port_map: RwLock::new(HashMap::new()),
            last_refresh: RwLock::new(Instant::now() - std::time::Duration::from_secs(60)),
        };
        instance.refresh();
        instance
    }

    /// Look up the owning PID for a local/remote socket.
    #[allow(dead_code)]
    pub fn lookup(&self, addr: &SocketAddr) -> Option<u32> {
        // Quick lookup in address map
        if let Some(pid) = self.addr_map.read().unwrap().get(addr).copied() {
            return Some(pid);
        }
        // Fallback to port-only lookup
        self.port_map.read().unwrap().get(&addr.port()).copied()
    }

    /// Look up by port number directly (pure read, see `lookup`).
    #[allow(dead_code)]
    pub fn lookup_port(&self, port: u16) -> Option<u32> {
        self.port_map.read().unwrap().get(&port).copied()
    }

    /// Refresh the table from the OS.
    pub fn refresh(&self) {
        let mut new_addr_map = HashMap::new();
        let mut new_port_map = HashMap::new();

        // 1) TCP table
        let mut size: u32 = 0;
        unsafe {
            GetExtendedTcpTable(
                std::ptr::null_mut(),
                &mut size,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );
        }

        if size > 0 {
            let mut buf = vec![0u8; size as usize];
            let ret = unsafe {
                GetExtendedTcpTable(
                    buf.as_mut_ptr() as *mut _,
                    &mut size,
                    0,
                    AF_INET as u32,
                    TCP_TABLE_OWNER_PID_ALL,
                    0,
                )
            };
            if ret == 0 && buf.len() >= 4 {
                let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                let row_size = std::mem::size_of::<MibTcpRowOwnerPid>();
                let mut offset = 4;
                for _ in 0..num_entries {
                    if offset + row_size > buf.len() {
                        break;
                    }
                    let row = unsafe { &*(buf.as_ptr().add(offset) as *const MibTcpRowOwnerPid) };
                    let local_port = u16::from_be((row.dw_local_port & 0xFFFF) as u16);
                    let local_ip = Ipv4Addr::from(u32::from_ne_bytes(row.dw_local_addr.to_ne_bytes()));
                    let pid = row.dw_owning_pid;

                    if pid > 0 {
                        new_addr_map.insert(SocketAddr::new(IpAddr::V4(local_ip), local_port), pid);
                        new_port_map.insert(local_port, pid);
                    }
                    offset += row_size;
                }
            }
        }

        // 2) UDP table
        let mut size: u32 = 0;
        unsafe {
            GetExtendedUdpTable(
                std::ptr::null_mut(),
                &mut size,
                0,
                AF_INET as u32,
                UDP_TABLE_OWNER_PID,
                0,
            );
        }

        if size > 0 {
            let mut buf = vec![0u8; size as usize];
            let ret = unsafe {
                GetExtendedUdpTable(
                    buf.as_mut_ptr() as *mut _,
                    &mut size,
                    0,
                    AF_INET as u32,
                    UDP_TABLE_OWNER_PID,
                    0,
                )
            };
            if ret == 0 && buf.len() >= 4 {
                let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
                let row_size = std::mem::size_of::<MibUdpRowOwnerPid>();
                let mut offset = 4;
                for _ in 0..num_entries {
                    if offset + row_size > buf.len() {
                        break;
                    }
                    let row = unsafe { &*(buf.as_ptr().add(offset) as *const MibUdpRowOwnerPid) };
                    let local_port = u16::from_be((row.dw_local_port & 0xFFFF) as u16);
                    let local_ip = Ipv4Addr::from(u32::from_ne_bytes(row.dw_local_addr.to_ne_bytes()));
                    let pid = row.dw_owning_pid;

                    if pid > 0 {
                        new_addr_map.insert(SocketAddr::new(IpAddr::V4(local_ip), local_port), pid);
                        new_port_map.insert(local_port, pid);
                    }
                    offset += row_size;
                }
            }
        }

        *self.addr_map.write().unwrap() = new_addr_map;
        *self.port_map.write().unwrap() = new_port_map;
        *self.last_refresh.write().unwrap() = Instant::now();
    }
}

impl Default for PortPidMap {
    fn default() -> Self {
        Self::new()
    }
}

