//! ETW (Event Tracing for Windows) consumer for the Kernel-Network provider.
//!
//! `provider` holds the provider GUID, `session` owns the real-time trace
//! session and forwards decoded [`NetworkEvent`]s over an `mpsc` channel, and
//! `decoder` turns raw `EVENT_RECORD`s into [`NetworkEvent`]s.

pub mod decoder;
pub mod provider;
pub mod session;

pub use session::Session;

use crate::models::Direction;

/// A single decoded network event (TCP/UDP send or receive).
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub timestamp: std::time::SystemTime,
    pub pid: u32,
    pub direction: Direction,
    pub size: u32,
    pub local_addr: std::net::SocketAddr,
    pub remote_addr: std::net::SocketAddr,
}

impl NetworkEvent {
    pub fn new(
        pid: u32,
        direction: Direction,
        size: u32,
        local_addr: std::net::SocketAddr,
        remote_addr: std::net::SocketAddr,
    ) -> Self {
        NetworkEvent {
            timestamp: std::time::SystemTime::now(),
            pid,
            direction,
            size,
            local_addr,
            remote_addr,
        }
    }
}
