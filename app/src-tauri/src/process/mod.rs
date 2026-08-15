//! Process metadata and network socket mapping.

pub mod icon;
pub mod info;
pub mod port_map;

pub use icon::get_process_icon_b64;
pub use info::Resolver;
pub use port_map::PortPidMap;
