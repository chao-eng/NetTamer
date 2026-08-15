//! Throttle table (runtime token buckets) and the persistence-aware manager.

pub mod manager;
pub use manager::Manager;

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use crate::models::Policy;
use crate::windivert::TokenBucket;

/// Runtime mapping of PID -> token bucket plus the authoritative policy set.
///
/// Buckets are keyed by PID. Policy lookup by PID is indirect: we keep a
/// `process_name -> PID` hint table (`name_pid`) populated both when a policy is
/// applied (if the PID is already known) and when the WinDivert engine observes
/// a live process (`note_process`).
pub struct ThrottleTable {
    buckets: Mutex<HashMap<u32, TokenBucket>>,
    policies: RwLock<HashMap<String, Policy>>,
    name_pid: Mutex<HashMap<String, u32>>,
}

impl ThrottleTable {
    pub fn new() -> Self {
        ThrottleTable {
            buckets: Mutex::new(HashMap::new()),
            policies: RwLock::new(HashMap::new()),
            name_pid: Mutex::new(HashMap::new()),
        }
    }

    /// Store a policy and update existing buckets.
    pub fn apply_policy(&self, p: Policy) {
        self.policies.write().unwrap().insert(p.id.clone(), p.clone());
        let mut buckets = self.buckets.lock().unwrap();
        // Update all buckets that might belong to this policy
        for bucket in buckets.values_mut() {
            if p.active && p.rate_limit_bps > 0 {
                bucket.set_rate(p.rate_limit_bps);
            }
        }
    }

    /// Bind a policy (by id) to a concrete PID and (re)create its bucket.
    #[allow(dead_code)]
    pub fn set_pid_for_policy(&self, policy_id: &str, pid: u32) {
        let pol = self.policies.read().unwrap().get(policy_id).cloned();
        if let Some(p) = pol {
            self.name_pid
                .lock()
                .unwrap()
                .insert(p.process_name.clone(), pid);
            self.ensure_bucket(pid, &p);
        }
    }

    /// Called by the engine when a process name is observed for a PID.
    pub fn note_process(&self, name: &str, pid: u32) {
        self.name_pid
            .lock()
            .unwrap()
            .insert(name.to_string(), pid);
    }

    /// Lookup policy by process name (case-insensitive, ignores `.exe` difference).
    pub fn policy_for_name(&self, name: &str) -> Option<Policy> {
        let name_lower = name.to_lowercase();
        self.policies
            .read()
            .unwrap()
            .values()
            .find(|p| {
                if !p.active {
                    return false;
                }
                let pn = p.process_name.to_lowercase();
                pn == name_lower
                    || format!("{}.exe", pn) == name_lower
                    || pn == format!("{}.exe", name_lower)
            })
            .cloned()
    }

    /// Admit `bytes` for `pid` under `policy`.
    pub fn admit(&self, pid: u32, policy: &Policy, bytes: usize) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(pid)
            .or_insert_with(|| TokenBucket::new(policy.rate_limit_bps));
        bucket.try_consume(bytes)
    }

    /// Return the active policy (if any) bound to `pid`.
    #[allow(dead_code)]
    pub fn policy_for_pid(&self, pid: u32) -> Option<Policy> {
        let name_pid = self.name_pid.lock().unwrap();
        let name = name_pid.iter().find(|(_, &v)| v == pid).map(|(k, _)| k.clone())?;
        drop(name_pid);
        self.policy_for_name(&name)
    }

    /// Remove a policy and its bucket.
    pub fn remove_policy(&self, id: &str) {
        let pol = self.policies.write().unwrap().remove(id);
        if let Some(p) = pol {
            if let Some(pid) = self.name_pid.lock().unwrap().get(&p.process_name).copied() {
                self.buckets.lock().unwrap().remove(&pid);
            }
        }
    }

    /// List all stored policies.
    #[allow(dead_code)]
    pub fn list_policies(&self) -> Vec<Policy> {
        self.policies.read().unwrap().values().cloned().collect()
    }

    #[allow(dead_code)]
    fn ensure_bucket(&self, pid: u32, p: &Policy) {
        let mut buckets = self.buckets.lock().unwrap();
        if p.active && p.rate_limit_bps > 0 {
            buckets.insert(pid, TokenBucket::new(p.rate_limit_bps));
        } else {
            buckets.remove(&pid);
        }
    }
}

impl Default for ThrottleTable {
    fn default() -> Self {
        Self::new()
    }
}
