//! Best-effort decoder for `Microsoft-Windows-Kernel-Network` events.
//!
//! The provider `{7dd42a49-5329-4832-8dfd-43d979153a88}` emits TcpIp/UdpIp
//! Send and Recv events. The `UserData` layout for IPv4 events is documented as:
//!
//! ```text
//! [PID: u32] [size: u32] [daddr: u32] [saddr: u32] [dport: u16] [sport: u16] ...
//! ```
//!
//! However, on some Windows builds the PID field is omitted from UserData
//! (available only via `EventHeader.ProcessId`), shifting all fields by −4.
//! We detect the layout at runtime by comparing `UserData[0..4]` against the
//! header PID.

use windows_sys::Win32::System::Diagnostics::Etw::EVENT_RECORD;

use crate::etw::NetworkEvent;
use crate::models::Direction;

/// Event IDs for the Kernel-Network provider (IPv4).
const EID_TCP_SEND_V4: u16 = 10;
const EID_TCP_RECV_V4: u16 = 11;
const EID_UDP_SEND_V4: u16 = 12;
const EID_UDP_RECV_V4: u16 = 13;

/// Event IDs for IPv6 variants.
const EID_TCP_SEND_V6: u16 = 26;
const EID_TCP_RECV_V6: u16 = 27;
const EID_UDP_SEND_V6: u16 = 28;
const EID_UDP_RECV_V6: u16 = 29;

/// Decode a single ETW record into a [`NetworkEvent`].
pub fn decode(event: &EVENT_RECORD) -> Option<NetworkEvent> {
    let header = &event.EventHeader;
    let pid = header.ProcessId;
    let eid = header.EventDescriptor.Id;

    let direction = match eid {
        EID_TCP_SEND_V4 | EID_UDP_SEND_V4 | EID_TCP_SEND_V6 | EID_UDP_SEND_V6 => Direction::Send,
        EID_TCP_RECV_V4 | EID_UDP_RECV_V4 | EID_TCP_RECV_V6 | EID_UDP_RECV_V6 => Direction::Recv,
        _ => return None,
    };

    let size = parse_size(event, pid);

    let local = socketaddr_unspecified();
    let remote = socketaddr_unspecified();

    Some(NetworkEvent::new(pid, direction, size, local, remote))
}

/// Try to extract the transfer `size` from the event's `UserData`.
///
/// Layout A (with PID prefix): `[PID:4][size:4][...]`
/// Layout B (no PID prefix):   `[size:4][...]`
///
/// We auto-detect by checking whether the first u32 equals the header PID.
fn parse_size(event: &EVENT_RECORD, header_pid: u32) -> u32 {
    if event.UserData.is_null() || event.UserDataLength < 4 {
        return 0;
    }

    let len = event.UserDataLength as usize;
    let data = unsafe { std::slice::from_raw_parts(event.UserData as *const u8, len) };

    let first = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);

    if len >= 8 {
        let second = u32::from_ne_bytes([data[4], data[5], data[6], data[7]]);

        // Layout A: first field matches header PID → size is the second field.
        if first == header_pid && header_pid != 0 {
            return second;
        }

        // Layout B: first field does NOT match PID → treat it as the size itself,
        // but only if it looks like a plausible byte count (< 64 KiB for a single
        // event). If *both* first and second are plausible, prefer first.
        if first > 0 && first <= 0xFFFF {
            return first;
        }
        if second > 0 && second <= 0xFFFF {
            return second;
        }

        // Fall back: return whichever is non-zero.
        if first > 0 {
            return first;
        }
        return second;
    }

    // Only 4 bytes available — treat as size directly.
    first
}

fn socketaddr_unspecified() -> std::net::SocketAddr {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
}

