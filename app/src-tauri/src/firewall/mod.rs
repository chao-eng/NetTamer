//! Firewall rule table and persistence-aware manager for process network control.

pub mod manager;
pub use manager::Manager;

use std::collections::HashMap;
use std::sync::RwLock;

use crate::models::FirewallRule;

/// In-memory table managing firewall rules for processes.
pub struct FirewallTable {
    rules: RwLock<HashMap<String, FirewallRule>>,
}

impl FirewallTable {
    pub fn new() -> Self {
        FirewallTable {
            rules: RwLock::new(HashMap::new()),
        }
    }

    /// Store or update a firewall rule.
    pub fn apply_rule(&self, rule: FirewallRule) {
        self.rules.write().unwrap().insert(rule.id.clone(), rule);
    }

    /// Lookup rule by process name (case-insensitive, ignores `.exe` difference).
    #[allow(dead_code)]
    pub fn rule_for_name(&self, name: &str) -> Option<FirewallRule> {
        let name_lower = name.to_lowercase();
        self.rules
            .read()
            .unwrap()
            .values()
            .find(|r| {
                if !r.active {
                    return false;
                }
                let pn = r.process_name.to_lowercase();
                pn == name_lower
                    || format!("{}.exe", pn) == name_lower
                    || pn == format!("{}.exe", name_lower)
            })
            .cloned()
    }

    /// Remove a rule by ID.
    pub fn remove_rule(&self, id: &str) {
        self.rules.write().unwrap().remove(id);
    }

    /// List all stored rules.
    #[allow(dead_code)]
    pub fn list_rules(&self) -> Vec<FirewallRule> {
        self.rules.read().unwrap().values().cloned().collect()
    }
}

impl Default for FirewallTable {
    fn default() -> Self {
        Self::new()
    }
}
