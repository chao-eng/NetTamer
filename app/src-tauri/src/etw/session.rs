//! Real-time ETW trace session for `Microsoft-Windows-Kernel-Network`.
//!
//! `start()` creates the kernel session, enables the provider, and spawns a
//! worker thread that consumes the real-time feed. Decoded [`NetworkEvent`]s
//! are pushed over an `mpsc::Receiver` returned to the caller, who is expected
//! to drain it into the [`crate::monitor::Aggregator`].
//!
//! NOTE / TODO: the `WNODE_HEADER` / `EVENT_TRACE_PROPERTIES` field assignments
//! and the `EVENT_TRACE_LOGFILEW.Anonymous.EventCallback` union access must be
//! verified against the exact `windows-sys = 0.52` layout on the build machine.
//! Event *payload* decoding (size + sockets) lives in `decoder.rs` and is also
//! flagged for verification.

use std::cell::RefCell;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use windows_sys::Win32::System::Diagnostics::Etw::{
    CloseTrace, EnableTraceEx2, OpenTraceW, ProcessTrace, StartTraceW, StopTraceW,
    CONTROLTRACE_HANDLE, EVENT_RECORD, EVENT_TRACE_LOGFILEW, EVENT_TRACE_PROPERTIES,
};

use crate::etw::decoder;
use crate::etw::provider::KERNEL_NETWORK_PROVIDER;
use crate::etw::NetworkEvent;
use crate::models::Error;

// --- Constants (mirroring the winmeta / evntrace definitions) ----------------
const EVENT_TRACE_REAL_TIME_MODE: u32 = 0x0000_0100;
const WNODE_FLAG_TRACED_GUID: u32 = 0x0002_0000;
const PROCESS_TRACE_MODE_REAL_TIME: u32 = 0x0000_0100;
const PROCESS_TRACE_MODE_EVENT_RECORD: u32 = 0x1000_0000;
const EVENT_CONTROL_CODE_ENABLE_PROVIDER: u32 = 1;
const TRACE_LEVEL_INFORMATION: u8 = 4;

const SESSION_NAME: &str = "NetTamerETW";

thread_local! {
    /// Sender handed to the worker thread so the C callback can forward events.
    static EVENT_TX: RefCell<Option<mpsc::Sender<NetworkEvent>>> = RefCell::new(None);
}

/// Real-time ETW session handle wrapper. Stopping it ends the worker thread
/// (which drops the internal sender) and closes the trace.
pub struct Session {
    name: String,
    worker: Option<JoinHandle<()>>,
}

/// Allocation backing `EVENT_TRACE_PROPERTIES` plus the trailing logger-name
/// wide-char buffer (required by the ETW API).
#[repr(C)]
struct TraceProps {
    props: EVENT_TRACE_PROPERTIES,
    name_buf: [u16; 256],
}

unsafe extern "system" fn event_callback(record: *mut EVENT_RECORD) {
    let record = &*record;
    if let Some(ev) = decoder::decode(record) {
        EVENT_TX.with(|cell| {
            let borrowed = cell.borrow();
            if let Some(tx) = borrowed.as_ref() {
                let _ = tx.send(ev);
            }
        });
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn make_props(name: &str, buffer_size: u32) -> TraceProps {
    let mut p: TraceProps = unsafe { std::mem::zeroed() };
    p.props.Wnode.BufferSize = std::mem::size_of::<TraceProps>() as u32;
    p.props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    p.props.BufferSize = buffer_size;
    p.props.LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
    p.props.LoggerNameOffset = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() as u32;
    p.props.LogFileNameOffset = 0;

    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let n = wide.len().min(p.name_buf.len());
    p.name_buf[..n].copy_from_slice(&wide[..n]);
    p
}

impl Session {
    /// Create and start the real-time kernel-network trace session.
    ///
    /// Returns the session plus a receiver that yields decoded network events.
    pub fn start(buffer_size: u32) -> Result<(Session, mpsc::Receiver<NetworkEvent>), Error> {
        let (tx, rx) = mpsc::channel::<NetworkEvent>();

        // 1) Create the trace session (cleanup stale session if exists).
        let mut props = make_props(SESSION_NAME, buffer_size);
        let _ = unsafe {
            StopTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                to_wide(SESSION_NAME).as_ptr(),
                &mut props.props as *mut _,
            )
        };

        let mut props = make_props(SESSION_NAME, buffer_size);
        let mut session_handle: CONTROLTRACE_HANDLE = unsafe { std::mem::zeroed() };
        let status = unsafe {
            StartTraceW(
                &mut session_handle,
                to_wide(SESSION_NAME).as_ptr(),
                &mut props.props as *mut _,
            )
        };
        if status != 0 && session_handle.Value == 0 {
            log::warn!("StartTraceW returned status code: {status}");
        }

        // 2) Enable the kernel-network provider on the session.
        let mut params: windows_sys::Win32::System::Diagnostics::Etw::ENABLE_TRACE_PARAMETERS =
            unsafe { std::mem::zeroed() };
        params.Version = 1;
        let _status = unsafe {
            EnableTraceEx2(
                session_handle,
                &KERNEL_NETWORK_PROVIDER,
                EVENT_CONTROL_CODE_ENABLE_PROVIDER,
                TRACE_LEVEL_INFORMATION,
                0xFFFF_FFFF_FFFF_FFFF, // match any keyword
                0,
                0,
                &mut params,
            )
        };

        // 3) Spawn the consumer thread.
        let worker = thread::spawn(move || {
            EVENT_TX.with(|cell| *cell.borrow_mut() = Some(tx));

            let mut logfile: EVENT_TRACE_LOGFILEW = unsafe { std::mem::zeroed() };
            let wide = to_wide(SESSION_NAME);
            logfile.LoggerName = wide.as_ptr() as *mut _;
            logfile.Anonymous1.ProcessTraceMode = PROCESS_TRACE_MODE_REAL_TIME | PROCESS_TRACE_MODE_EVENT_RECORD;
            logfile.Anonymous2.EventRecordCallback = Some(event_callback);

            let consumer = unsafe { OpenTraceW(&mut logfile) };
            if consumer.Value == u64::MAX {
                EVENT_TX.with(|cell| *cell.borrow_mut() = None);
                return;
            }

            let mut handle = consumer;
            unsafe {
                ProcessTrace(&mut handle, 1, std::ptr::null(), std::ptr::null());
            }
            unsafe {
                CloseTrace(consumer);
            }
            EVENT_TX.with(|cell| *cell.borrow_mut() = None);
        });

        Ok((Session { name: SESSION_NAME.to_string(), worker: Some(worker) }, rx))
    }

    /// Stop the trace session and join the consumer thread.
    pub fn stop(mut self) -> Result<(), Error> {
        let mut props = make_props(&self.name, 0);
        let _ = unsafe {
            StopTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                to_wide(&self.name).as_ptr(),
                &mut props.props as *mut _,
            )
        };
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        Ok(())
    }
}
