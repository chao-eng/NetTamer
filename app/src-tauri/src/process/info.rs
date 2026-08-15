//! Best-effort process metadata resolver with a small LRU-ish cache.
//!
//! Uses `OpenProcess` + `QueryFullProcessImageNameW` for the executable path,
//! extracting the file name as the process name. Falls back to a snapshot via
//! `CreateToolhelp32Snapshot` for processes that deny `PROCESS_QUERY_LIMITED_INFORMATION`.

use std::collections::HashMap;
use std::sync::Mutex;

/// Resolved metadata for a process.
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub pid: u32,
    pub name: String,
    pub path: String,
    /// Base64-encoded process icon (empty placeholder for now).
    pub icon_b64: String,
    pub user: String,
}

/// Caches process metadata to avoid repeated OS lookups on the hot path.
pub struct Resolver {
    cache: Mutex<HashMap<u32, Info>>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Resolve metadata for `pid`, consulting the cache first.
    pub fn resolve(&self, pid: u32) -> Info {
        if let Some(info) = self.cache.lock().unwrap().get(&pid) {
            return info.clone();
        }
        let info = Self::query_process(pid);
        self.cache.lock().unwrap().insert(pid, info.clone());
        info
    }

    /// Drop a cached entry (e.g. on process exit).
    pub fn invalidate(&self, pid: u32) {
        self.cache.lock().unwrap().remove(&pid);
    }

    /// Query the OS for process name and path via Win32 APIs.
    fn query_process(pid: u32) -> Info {
        // PID 0 (System Idle) and PID 4 (System) are special.
        if pid == 0 {
            return Info {
                pid,
                name: "System Idle Process".to_string(),
                ..Default::default()
            };
        }
        if pid == 4 {
            return Info {
                pid,
                name: "System".to_string(),
                ..Default::default()
            };
        }

        // Try QueryFullProcessImageNameW first.
        if let Some(path) = Self::query_image_name(pid) {
            let name = path
                .rsplit('\\')
                .next()
                .unwrap_or(&path)
                .to_string();
            return Info {
                pid,
                name,
                path,
                icon_b64: String::new(),
                user: String::new(),
            };
        }

        // Fallback: try toolhelp snapshot.
        if let Some(name) = Self::query_via_snapshot(pid) {
            return Info {
                pid,
                name,
                path: String::new(),
                icon_b64: String::new(),
                user: String::new(),
            };
        }

        // Last resort: use PID-based placeholder.
        Info {
            pid,
            name: format!("pid_{}", pid),
            ..Default::default()
        }
    }

    /// Use `OpenProcess` + `QueryFullProcessImageNameW` to get the full image path.
    fn query_image_name(pid: u32) -> Option<String> {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 {
                return None;
            }

            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
            CloseHandle(handle);

            if ok == 0 || len == 0 {
                return None;
            }

            Some(String::from_utf16_lossy(&buf[..len as usize]))
        }
    }

    /// Fall back to `CreateToolhelp32Snapshot` + `Process32FirstW` / `Process32NextW`.
    fn query_via_snapshot(pid: u32) -> Option<String> {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };

        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap == INVALID_HANDLE_VALUE || snap == 0 {
                return None;
            }

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snap, &mut entry) == 0 {
                CloseHandle(snap);
                return None;
            }

            loop {
                if entry.th32ProcessID == pid {
                    CloseHandle(snap);
                    let name_len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    return Some(String::from_utf16_lossy(&entry.szExeFile[..name_len]));
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }

            CloseHandle(snap);
            None
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

