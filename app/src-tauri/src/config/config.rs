//! Typed views over the persisted KV configuration.

use std::collections::HashMap;

use crate::models::Error;
use crate::store::{ConfigStore, Db};

/// Wraps the `config` table with typed get/set helpers.
pub struct Config {
    db: Db,
}

impl Config {
    pub fn new(db: Db) -> Self {
        Config { db }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.db.config_get(key).ok().flatten()
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), Error> {
        self.db.config_set(key, value)
    }

    pub fn get_all(&self) -> HashMap<String, String> {
        self.db.config_all().unwrap_or_default()
    }

    /// Idempotently insert the project's default configuration values.
    pub fn ensure_defaults(&self) {
        for (k, v) in Self::defaults() {
            let _ = self.db.config_set_if_missing(k, v);
        }
    }

    fn defaults() -> Vec<(&'static str, &'static str)> {
        vec![
            ("refresh_interval_ms", "1000"),
            ("theme", "dark"),
            ("auto_start", "false"),
            ("minimize_to_tray", "true"),
            ("alert_sound", "true"),
            ("taskbar_speed", "false"),
        ]
    }
}
