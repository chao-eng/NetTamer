//! Best-effort process metadata resolver with a small LRU-ish cache.
//!
//! Uses `OpenProcess` + `QueryFullProcessImageNameW` for the executable path,
//! extracting the file name as the process name. Falls back to a snapshot via
//! `CreateToolhelp32Snapshot` for processes that deny `PROCESS_QUERY_LIMITED_INFORMATION`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::models::ProcessCategory;

/// Resolved metadata for a process.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Info {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub category: ProcessCategory,
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
    #[allow(dead_code)]
    pub fn invalidate(&self, pid: u32) {
        self.cache.lock().unwrap().remove(&pid);
    }

    /// Find all executable paths for running processes matching the given process name.
    /// Supports matching "chrome.exe", "chrome", or full path.
    pub fn find_exe_paths_by_name(&self, process_name: &str) -> Vec<String> {
        let trimmed = process_name.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        // If it's already an absolute existing file path, return it directly.
        if std::path::Path::new(trimmed).is_absolute() {
            return vec![trimmed.to_string()];
        }

        let target_lower = trimmed.to_lowercase();
        let target_with_exe = if target_lower.ends_with(".exe") {
            target_lower.clone()
        } else {
            format!("{}.exe", target_lower)
        };

        let mut paths = Vec::new();

        // 1. Check existing cache first
        {
            let cache = self.cache.lock().unwrap();
            for info in cache.values() {
                let name_lower = info.name.to_lowercase();
                if (name_lower == target_lower || name_lower == target_with_exe) && !info.path.is_empty() {
                    paths.push(info.path.clone());
                }
            }
        }

        // 2. Enumerate live processes snapshot to find current PIDs
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };

        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap != INVALID_HANDLE_VALUE && snap != 0 {
                let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

                if Process32FirstW(snap, &mut entry) != 0 {
                    loop {
                        let name_len = entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(entry.szExeFile.len());
                        let exe_name = String::from_utf16_lossy(&entry.szExeFile[..name_len]).to_lowercase();

                        if exe_name == target_lower || exe_name == target_with_exe {
                            if let Some(path) = Self::query_image_name(entry.th32ProcessID) {
                                paths.push(path);
                            }
                        }

                        if Process32NextW(snap, &mut entry) == 0 {
                            break;
                        }
                    }
                }
                CloseHandle(snap);
            }
        }

        // 3. Fallback: query common paths and environment if not found in running processes
        if paths.is_empty() {
            if let Some(path) = Self::find_installed_exe_path(&target_with_exe) {
                paths.push(path);
            }
        }

        paths.sort();
        paths.dedup();
        paths
    }

    /// Search common install directories and App Paths registry for the executable.
    fn find_installed_exe_path(exe_name: &str) -> Option<String> {
        // A. Check common paths
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());

        let candidates = [
            format!("{}\\Google\\Chrome\\Application\\{}", program_files, exe_name),
            format!("{}\\Google\\Chrome\\Application\\{}", program_files_x86, exe_name),
            format!("{}\\Google\\Chrome\\Application\\{}", local_app_data, exe_name),
            format!("{}\\Microsoft\\Edge\\Application\\{}", program_files, exe_name),
            format!("{}\\Microsoft\\Edge\\Application\\{}", program_files_x86, exe_name),
            format!("{}\\Steam\\{}", program_files_x86, exe_name),
            format!("{}\\Steam\\{}", program_files, exe_name),
            format!("{}\\Tencent\\WeChat\\{}", program_files, exe_name),
            format!("{}\\Tencent\\WeChat\\{}", program_files_x86, exe_name),
            format!("{}\\System32\\{}", sys_root, exe_name),
            format!("{}\\{}", sys_root, exe_name),
        ];

        for cand in candidates {
            if std::path::Path::new(&cand).is_file() {
                return Some(cand);
            }
        }

        // B. Check PATH environment variable
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let full = dir.join(exe_name);
                if full.is_file() {
                    if let Some(s) = full.to_str() {
                        return Some(s.to_string());
                    }
                }
            }
        }

        None
    }

    /// Query the OS for process name, path and category via Win32 APIs.
    fn query_process(pid: u32) -> Info {
        // PID 0 (System Idle) and PID 4 (System) are fixed kernel/system processes in Windows.
        if pid == 0 {
            return Info {
                pid,
                name: "System Idle Process".to_string(),
                path: "[Windows Kernel]".to_string(),
                category: ProcessCategory::Kernel,
                ..Default::default()
            };
        }
        if pid == 4 {
            return Info {
                pid,
                name: "Windows 系统网络 (System)".to_string(),
                path: "[Windows Kernel]".to_string(),
                category: ProcessCategory::Kernel,
                ..Default::default()
            };
        }

        // Try QueryFullProcessImageNameW first.
        let (name, path) = if let Some(p) = Self::query_image_name(pid) {
            let n = p.rsplit('\\').next().unwrap_or(&p).to_string();
            (n, p)
        } else if let Some(n) = Self::query_via_snapshot(pid) {
            (n, String::new())
        } else {
            (format!("pid_{}", pid), String::new())
        };

        let category = Self::classify_process(pid, &name, &path);

        Info {
            pid,
            name,
            path,
            category,
            icon_b64: String::new(),
            user: String::new(),
        }
    }

    /// Classify a process into Kernel, WindowsService, or UserApp.
    fn classify_process(pid: u32, name: &str, path: &str) -> ProcessCategory {
        if pid == 0 || pid == 4 {
            return ProcessCategory::Kernel;
        }

        let name_lower = name.to_lowercase();
        let path_lower = path.to_lowercase();

        // 1. Known Windows built-in core system services
        let known_windows_services = [
            "svchost.exe",
            "services.exe",
            "lsass.exe",
            "csrss.exe",
            "wininit.exe",
            "smss.exe",
            "spoolsv.exe",
            "dwm.exe",
            "fontdrvhost.exe",
            "sihost.exe",
            "ctfmon.exe",
            "taskhostw.exe",
            "runtimebroker.exe",
            "searchhost.exe",
            "startmenuexperiencehost.exe",
            "searchindexer.exe",
            "audiodg.exe",
            "wlanext.exe",
            "dashost.exe",
            "explorer.exe",
        ];

        if known_windows_services.contains(&name_lower.as_str()) {
            return ProcessCategory::WindowsService;
        }

        // 2. Check Windows Session ID (Session 0 is dedicated to Windows Services).
        #[link(name = "kernel32")]
        extern "system" {
            fn ProcessIdToSessionId(process_id: u32, session_id: *mut u32) -> i32;
        }

        let mut session_id: u32 = 1;
        let has_session = unsafe { ProcessIdToSessionId(pid, &mut session_id) };

        // Must be located in C:\Windows\ AND in Session 0 to be a Windows native system service.
        // Third-party background services (e.g. verge-mihomo, MySQL, Docker) belong to UserApp.
        let is_in_windows_dir = path_lower.starts_with("c:\\windows\\")
            || path_lower.contains("\\windows\\system32\\")
            || path_lower.contains("\\windows\\syswow64\\")
            || path_lower.contains("\\windows\\systemapps\\");

        if is_in_windows_dir && has_session != 0 && session_id == 0 {
            return ProcessCategory::WindowsService;
        }

        ProcessCategory::UserApp
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

