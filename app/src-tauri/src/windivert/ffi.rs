//! Low-level FFI bindings to the WinDivert C API (WinDivert 1.4+).
//!
//! We link against the import library `WinDivert.lib` (shipped with the
//! WinDivert SDK) via `#[link(name = "WinDivert")]`. At runtime the directory
//! containing this binary must also contain `WinDivert.dll` and `WinDivert.sys`
//! (the signed kernel driver), otherwise `WinDivertOpen` returns
//! `INVALID_HANDLE`.
//!
//! TODO: verify the exact `WinDivertOpen` calling convention / argument types
//! against the installed WinDivert version (1.4.x uses an ANSI `const char*`
//! filter string, not a wide string).

#![allow(dead_code, non_snake_case, non_camel_case_types)]

use std::os::raw::c_void;

/// WinDivert packet address (mirrors `WINDIVERT_ADDRESS`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WINDIVERT_ADDRESS {
    pub if_idx: u32,
    pub sub_if_idx: u32,
    /// 0 = outbound (send), 1 = inbound (recv).
    pub direction: u8,
}

pub const WINDIVERT_LAYER_NETWORK: u32 = 0;
pub const WINDIVERT_LAYER_NETWORK_FORWARD: u32 = 1;

pub const WINDIVERT_FLAG_SNIFF: u64 = 0x0001;
pub const WINDIVERT_FLAG_DROP: u64 = 0x0002;

pub const WINDIVERT_PARAM_QUEUE_LEN: i32 = 0;
pub const WINDIVERT_PARAM_QUEUE_TIME: i32 = 1;

/// Sentinel handle returned on failure (`(HANDLE)-1`).
pub const INVALID_HANDLE: *mut c_void = usize::MAX as *mut c_void;

#[link(name = "WinDivert")]
extern "C" {
    /// `WinDivertOpen(filter, layer, priority, flags) -> HANDLE`.
    pub fn WinDivertOpen(
        filter: *const u8,
        layer: u32,
        priority: i16,
        flags: u64,
    ) -> *mut c_void;

    /// `WinDivertRecv(handle, packet, packetLen, addr, recvLen) -> BOOL`.
    pub fn WinDivertRecv(
        handle: *mut c_void,
        packet: *mut c_void,
        packet_len: u32,
        addr: *mut WINDIVERT_ADDRESS,
        recv_len: *mut u32,
    ) -> i32;

    /// `WinDivertSend(handle, packet, packetLen, addr, sendLen) -> BOOL`.
    pub fn WinDivertSend(
        handle: *mut c_void,
        packet: *const c_void,
        packet_len: u32,
        addr: *const WINDIVERT_ADDRESS,
        send_len: *mut u32,
    ) -> i32;

    /// `WinDivertClose(handle) -> BOOL`.
    pub fn WinDivertClose(handle: *mut c_void) -> i32;

    /// `WinDivertSetParam(handle, param, value) -> BOOL`.
    pub fn WinDivertSetParam(handle: *mut c_void, param: i32, value: u64) -> i32;
}
