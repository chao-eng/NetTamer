//! ETW provider identifiers used by NetTamer.

use windows_sys::core::GUID;

/// `Microsoft-Windows-Kernel-Network`
/// (`{7dd42a49-5329-4832-8dfd-43d979153a88}`). Emits the TCP/UDP Send/Recv
/// events we aggregate for per-process rate tracking.
pub const KERNEL_NETWORK_PROVIDER: GUID = GUID {
    data1: 0x7dd42a49,
    data2: 0x5329,
    data3: 0x4832,
    data4: [0x8d, 0xfd, 0x43, 0xd9, 0x79, 0x15, 0x3a, 0x88],
};
