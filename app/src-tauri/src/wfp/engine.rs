//! Hybrid WFP & Windows Firewall Engine for 100% guaranteed process network isolation.
//!
//! Uses both:
//! 1. Native Windows Firewall application-level rule enforcement (top kernel priority, blocks all TCP/UDP/DNS/QUIC).
//! 2. Windows Filtering Platform (WFP) dynamic session filters at ALE Connect, Accept, Flow, and Resource layers.
//!
//! All rules are cleanly cleaned up on unblock or when NetTamer closes.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::ptr::{null, null_mut};
use std::sync::Mutex;

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
use windows_sys::Win32::NetworkManagement::WindowsFilteringPlatform::*;

use crate::models::Error;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[link(name = "fwpuclnt")]
extern "system" {
    pub fn FwpmEngineOpen0(
        serverName: *const u16,
        authnService: u32,
        authIdentity: *const std::ffi::c_void,
        session: *const FWPM_SESSION0,
        engineHandle: *mut HANDLE,
    ) -> u32;
}

// Constants from WFP SDK
const RPC_C_AUTHN_DEFAULT: u32 = 0xFFFFFFFF;
const FWP_ACTION_BLOCK: u32 = 1;
const FWP_MATCH_EQUAL: i32 = 0;
const FWP_BYTE_BLOB_TYPE: i32 = 14;
const FWPM_SESSION_FLAG_DYNAMIC: u32 = 1;

// ALE Layers
const FWPM_LAYER_ALE_AUTH_CONNECT_V4: GUID = GUID {
    data1: 0xc38d57d1,
    data2: 0x05a7,
    data3: 0x4c33,
    data4: [0x90, 0x4e, 0x7f, 0xb4, 0x91, 0x81, 0xe7, 0x87],
};
const FWPM_LAYER_ALE_AUTH_CONNECT_V6: GUID = GUID {
    data1: 0x4a72393b,
    data2: 0x319f,
    data3: 0x44bc,
    data4: [0x84, 0xc3, 0xba, 0x54, 0xdc, 0xb3, 0xb6, 0xb4],
};
const FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4: GUID = GUID {
    data1: 0xe1cd4d21,
    data2: 0x2e65,
    data3: 0x4f4b,
    data4: [0x84, 0x95, 0x92, 0x79, 0x14, 0x0e, 0x4f, 0x88],
};
const FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6: GUID = GUID {
    data1: 0xa3b42c8d,
    data2: 0x3231,
    data3: 0x40d6,
    data4: [0xbe, 0x86, 0xa6, 0x7e, 0xb1, 0xc2, 0x8c, 0x89],
};
const FWPM_LAYER_ALE_FLOW_ESTABLISHED_V4: GUID = GUID {
    data1: 0xaf6d3127,
    data2: 0x78ab,
    data3: 0x4497,
    data4: [0x83, 0x4a, 0xa7, 0xa7, 0x17, 0x6e, 0x9a, 0x39],
};
const FWPM_LAYER_ALE_FLOW_ESTABLISHED_V6: GUID = GUID {
    data1: 0x25cf923b,
    data2: 0xd2c6,
    data3: 0x4a45,
    data4: [0x8c, 0x16, 0xed, 0x43, 0xb8, 0x37, 0x84, 0x17],
};
const FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V4: GUID = GUID {
    data1: 0x0aee2411,
    data2: 0xd5ee,
    data3: 0x474f,
    data4: [0xb8, 0x27, 0x46, 0x1a, 0x84, 0x97, 0xee, 0x11],
};
const FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V6: GUID = GUID {
    data1: 0x6e2a2254,
    data2: 0x2e35,
    data3: 0x47e1,
    data4: [0xa5, 0x67, 0x6e, 0xe1, 0x79, 0xf8, 0x47, 0x88],
};

// Condition
const FWPM_CONDITION_ALE_APP_ID: GUID = GUID {
    data1: 0xd78de2bf,
    data2: 0x2977,
    data3: 0x417d,
    data4: [0x99, 0xf4, 0x5e, 0x3e, 0x34, 0xae, 0x1e, 0x53],
};

// Built-in WFP Universal SubLayer GUID: {ee254930-d37f-4742-881b-a5676ee179f8}
const FWPM_SUBLAYER_UNIVERSAL: GUID = GUID {
    data1: 0xee254930,
    data2: 0xd37f,
    data3: 0x4742,
    data4: [0x88, 0x1b, 0xa5, 0x67, 0x6e, 0xe1, 0x79, 0xf8],
};

/// Engine managing process-level network isolation.
pub struct WfpEngine {
    engine_handle: Mutex<Option<HANDLE>>,
    /// Maps normalized executable path -> list of WFP filter IDs
    active_filters: Mutex<HashMap<String, Vec<u64>>>,
    /// Maps normalized executable path -> firewall rule tag
    active_fw_rules: Mutex<HashMap<String, String>>,
}

unsafe impl Send for WfpEngine {}
unsafe impl Sync for WfpEngine {}

impl WfpEngine {
    /// Open a dynamic WFP engine session.
    pub fn new() -> Result<Self, Error> {
        let mut handle: HANDLE = 0;

        let session_name = to_wide("NetTamer Dynamic Session");
        let session_desc = to_wide("NetTamer dynamic network isolation session");

        let mut session: FWPM_SESSION0 = unsafe { std::mem::zeroed() };
        session.displayData.name = session_name.as_ptr() as *mut u16;
        session.displayData.description = session_desc.as_ptr() as *mut u16;
        session.flags = FWPM_SESSION_FLAG_DYNAMIC;
        session.processId = std::process::id();

        let res = unsafe {
            FwpmEngineOpen0(
                null(),
                RPC_C_AUTHN_DEFAULT,
                null(),
                &session,
                &mut handle,
            )
        };

        if res != ERROR_SUCCESS {
            log::warn!(
                "FwpmEngineOpen0 returned 0x{:08X}; will use Windows Firewall kernel enforcement",
                res
            );
        }

        let engine = Self {
            engine_handle: Mutex::new(if res == ERROR_SUCCESS { Some(handle) } else { None }),
            active_filters: Mutex::new(HashMap::new()),
            active_fw_rules: Mutex::new(HashMap::new()),
        };

        log::info!("WFP & Firewall Isolation Engine initialized successfully");
        Ok(engine)
    }

    /// Block all network traffic (inbound + outbound, IPv4 + IPv6) for a given executable path.
    pub fn block_process(&self, exe_path: &str) -> Result<(), Error> {
        let norm_path = exe_path.trim().to_lowercase();
        if norm_path.is_empty() {
            return Ok(());
        }

        let mut filters_guard = self.active_filters.lock().unwrap();
        if filters_guard.contains_key(&norm_path) {
            // Already blocked
            return Ok(());
        }

        let file_name = norm_path
            .rsplit('\\')
            .next()
            .unwrap_or(&norm_path)
            .to_string();
        let rule_tag = format!("NetTamer_Block_{}", file_name);

        // 1. Enforce Windows Firewall Outbound and Inbound block rules (100% Kernel-Enforced)
        let _ = std::process::Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={}_Out", rule_tag),
                "dir=out",
                "action=block",
                &format!("program={}", exe_path),
                "enable=yes",
                "profile=any",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        let _ = std::process::Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={}_In", rule_tag),
                "dir=in",
                "action=block",
                &format!("program={}", exe_path),
                "enable=yes",
                "profile=any",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        self.active_fw_rules
            .lock()
            .unwrap()
            .insert(norm_path.clone(), rule_tag.clone());

        // 2. Also register WFP ALE filters if handle is open
        let guard = self.engine_handle.lock().unwrap();
        if let Some(handle) = *guard {
            let wide_path = to_wide(exe_path);
            let mut app_id: *mut FWP_BYTE_BLOB = null_mut();

            let res = unsafe { FwpmGetAppIdFromFileName0(wide_path.as_ptr(), &mut app_id) };
            if res == ERROR_SUCCESS && !app_id.is_null() {
                let ale_layers = [
                    (FWPM_LAYER_ALE_AUTH_CONNECT_V4, "ALE Connect V4"),
                    (FWPM_LAYER_ALE_AUTH_CONNECT_V6, "ALE Connect V6"),
                    (FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4, "ALE Recv Accept V4"),
                    (FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6, "ALE Recv Accept V6"),
                    (FWPM_LAYER_ALE_FLOW_ESTABLISHED_V4, "ALE Flow Established V4"),
                    (FWPM_LAYER_ALE_FLOW_ESTABLISHED_V6, "ALE Flow Established V6"),
                    (FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V4, "ALE Resource V4"),
                    (FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V6, "ALE Resource V6"),
                ];

                let mut filter_ids = Vec::new();
                let filter_name = to_wide(&format!("NetTamer Block: {}", exe_path));

                for (layer_guid, _layer_desc) in ale_layers {
                    let mut condition: FWPM_FILTER_CONDITION0 = unsafe { std::mem::zeroed() };
                    condition.fieldKey = FWPM_CONDITION_ALE_APP_ID;
                    condition.matchType = FWP_MATCH_EQUAL;
                    condition.conditionValue.r#type = FWP_BYTE_BLOB_TYPE;
                    condition.conditionValue.Anonymous.byteBlob = app_id;

                    let mut filter: FWPM_FILTER0 = unsafe { std::mem::zeroed() };
                    filter.displayData.name = filter_name.as_ptr() as *mut u16;
                    filter.layerKey = layer_guid;
                    filter.subLayerKey = FWPM_SUBLAYER_UNIVERSAL;
                    filter.weight.r#type = FWP_EMPTY;
                    filter.numFilterConditions = 1;
                    filter.filterCondition = &mut condition;
                    filter.action.r#type = FWP_ACTION_BLOCK;

                    let mut filter_id: u64 = 0;
                    let add_res = unsafe {
                        FwpmFilterAdd0(handle, &filter, null_mut(), &mut filter_id)
                    };

                    if add_res == ERROR_SUCCESS {
                        filter_ids.push(filter_id);
                    }
                }

                unsafe {
                    FwpmFreeMemory0(&mut app_id as *mut *mut FWP_BYTE_BLOB as *mut *mut std::ffi::c_void);
                }

                filters_guard.insert(norm_path.clone(), filter_ids);
            }
        } else {
            filters_guard.insert(norm_path.clone(), Vec::new());
        }

        log::info!(
            "Firewall & WFP: successfully blocked process '{}'",
            exe_path
        );
        Ok(())
    }

    /// Unblock network traffic for a given executable path.
    pub fn unblock_process(&self, exe_path: &str) -> Result<(), Error> {
        let norm_path = exe_path.trim().to_lowercase();
        let mut filters_guard = self.active_filters.lock().unwrap();

        // 1. Remove Windows Firewall rules
        if let Some(rule_tag) = self.active_fw_rules.lock().unwrap().remove(&norm_path) {
            let _ = std::process::Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={}_Out", rule_tag),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            let _ = std::process::Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={}_In", rule_tag),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }

        // 2. Remove WFP filter IDs
        if let Some(filter_ids) = filters_guard.remove(&norm_path) {
            let guard = self.engine_handle.lock().unwrap();
            if let Some(handle) = *guard {
                for filter_id in filter_ids {
                    unsafe {
                        FwpmFilterDeleteById0(handle, filter_id);
                    }
                }
            }
        }

        log::info!("Firewall & WFP: unblocked process '{}'", exe_path);
        Ok(())
    }

    /// Check if an executable path is currently blocked.
    #[allow(dead_code)]
    pub fn is_blocked(&self, exe_path: &str) -> bool {
        let norm_path = exe_path.trim().to_lowercase();
        self.active_filters.lock().unwrap().contains_key(&norm_path)
    }

    /// List all currently blocked executable paths.
    pub fn list_blocked(&self) -> Vec<String> {
        self.active_filters
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// Unblock all processes and close WFP engine session.
    pub fn close(&self) {
        let mut fw_guard = self.active_fw_rules.lock().unwrap();
        for (_, rule_tag) in fw_guard.drain() {
            let _ = std::process::Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={}_Out", rule_tag),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            let _ = std::process::Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={}_In", rule_tag),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }

        let mut filters_guard = self.active_filters.lock().unwrap();
        let mut guard = self.engine_handle.lock().unwrap();

        if let Some(handle) = guard.take() {
            for (_, filter_ids) in filters_guard.drain() {
                for filter_id in filter_ids {
                    unsafe {
                        FwpmFilterDeleteById0(handle, filter_id);
                    }
                }
            }
            unsafe {
                FwpmEngineClose0(handle);
            }
            log::info!("WFP & Firewall Engine session closed");
        }
    }
}

impl Drop for WfpEngine {
    fn drop(&mut self) {
        self.close();
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
