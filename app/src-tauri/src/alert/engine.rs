//! Alert rule evaluation engine.

use std::collections::HashMap;
use std::sync::{mpsc, RwLock, Mutex};
use std::time::{Duration, Instant};

use crate::alert::matcher;
use crate::models::{AlertEvent, AlertHistoryFilter, Error, ProcessStats, Rule};
use crate::store::{AlertStore, Db};

/// Holds the active rule set, per-key cooldown timers, and the channel used to
/// push fired [`AlertEvent`]s to the frontend (relayed as `alert:triggered`).
pub struct Engine {
    rules: RwLock<HashMap<String, Rule>>,
    cooldowns: Mutex<HashMap<String, Instant>>,
    alert_tx: mpsc::Sender<AlertEvent>,
    store: Db,
}

impl Engine {
    pub fn new(store: Db, alert_tx: mpsc::Sender<AlertEvent>) -> Self {
        Engine {
            rules: RwLock::new(HashMap::new()),
            cooldowns: Mutex::new(HashMap::new()),
            alert_tx,
            store,
        }
    }

    /// Load persisted rules into memory (called during setup).
    pub fn load_rules(&self) {
        if let Ok(rules) = self.store.list_rules() {
            let mut r = self.rules.write().unwrap();
            for rule in rules {
                r.insert(rule.id.clone(), rule);
            }
        }
    }

    /// Evaluate the current process snapshot against all enabled rules.
    pub fn evaluate(&self, stats: &[ProcessStats]) {
        let rules = self.rules.read().unwrap();
        let mut cooldowns = self.cooldowns.lock().unwrap();

        for stat in stats {
            for rule in rules.values() {
                if !rule.enabled {
                    continue;
                }
                if !matcher::match_process(&rule.process_name, &stat.name) {
                    continue;
                }

                let rate = match rule.direction {
                    0 => stat.upload_rate,
                    1 => stat.download_rate,
                    2 => stat.upload_rate.max(stat.download_rate),
                    _ => stat.upload_rate,
                };

                if rule.threshold <= 0.0 || rate <= rule.threshold {
                    continue;
                }

                let key = format!("{}:{}", rule.id, stat.name);
                let cooldown = Duration::from_secs(rule.cooldown_sec.max(0) as u64);
                if let Some(last) = cooldowns.get(&key) {
                    if last.elapsed() < cooldown {
                        continue;
                    }
                }
                cooldowns.insert(key, Instant::now());

                let ev_direction = match rule.direction {
                    0 => 0,
                    1 => 1,
                    2 => {
                        if stat.upload_rate >= stat.download_rate {
                            0
                        } else {
                            1
                        }
                    }
                    _ => 0,
                };

                let ev = AlertEvent {
                    id: crate::models::new_id(),
                    rule_id: rule.id.clone(),
                    process_name: stat.name.clone(),
                    pid: stat.pid,
                    direction: ev_direction,
                    current_rate: rate,
                    threshold: rule.threshold,
                    triggered_at: crate::models::now_secs(),
                };

                // Best-effort fan-out; ignore send errors (no listener yet).
                let _ = self.alert_tx.send(ev.clone());
                let _ = self.store.save_alert_event(&ev);
            }
        }
    }

    pub fn create_rule(&self, rule: Rule) -> Result<(), Error> {
        self.store.save_rule(&rule)?;
        self.rules.write().unwrap().insert(rule.id.clone(), rule);
        Ok(())
    }

    pub fn update_rule(&self, rule: Rule) -> Result<(), Error> {
        self.store.save_rule(&rule)?;
        self.rules.write().unwrap().insert(rule.id.clone(), rule);
        Ok(())
    }

    pub fn delete_rule(&self, id: &str) -> Result<(), Error> {
        self.store.delete_rule(id)?;
        self.rules.write().unwrap().remove(id);
        Ok(())
    }

    pub fn list_rules(&self) -> Vec<Rule> {
        self.rules.read().unwrap().values().cloned().collect()
    }

    pub fn get_history(&self, f: &AlertHistoryFilter) -> Result<Vec<AlertEvent>, Error> {
        self.store.list_alert_events(f)
    }
}
