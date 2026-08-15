//! Process metadata and the socket -> PID mapping used by WinDivert.

pub mod info;
pub mod port_map;

pub use info::Resolver;
pub use port_map::PortPidMap;
