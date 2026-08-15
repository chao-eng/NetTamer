//! Database schema bootstrap (architecture §10.1) plus default config rows.

use crate::models::Error;
use crate::store::Db;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS alert_rules (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    process_name  TEXT NOT NULL,
    threshold     REAL NOT NULL,
    direction     INTEGER DEFAULT 0,
    cooldown_sec  INTEGER DEFAULT 60,
    enabled       INTEGER DEFAULT 1,
    created_at    INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS alert_events (
    id            TEXT PRIMARY KEY,
    rule_id       TEXT NOT NULL,
    process_name  TEXT NOT NULL,
    pid           INTEGER NOT NULL,
    direction     INTEGER DEFAULT 0,
    current_rate  REAL NOT NULL,
    threshold     REAL NOT NULL,
    triggered_at  INTEGER NOT NULL,
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_alert_events_time ON alert_events(triggered_at DESC);

CREATE TABLE IF NOT EXISTS throttle_policies (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL UNIQUE,
    process_name   TEXT NOT NULL,
    rate_limit_bps INTEGER NOT NULL,
    limit_upload   INTEGER DEFAULT 1,
    limit_download INTEGER DEFAULT 1,
    active         INTEGER DEFAULT 1,
    created_at     INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

const DEFAULT_CONFIG: &[(&str, &str)] = &[
    ("refresh_interval_ms", "1000"),
    ("theme", "dark"),
    ("auto_start", "false"),
    ("minimize_to_tray", "true"),
    ("alert_sound", "true"),
];

/// Create tables (idempotent) and seed default config values (idempotent).
pub fn run(db: &Db) -> Result<(), Error> {
    db.execute_batch(SCHEMA)?;
    // Attempt column migration if table already existed without direction
    let _ = db.execute_batch("ALTER TABLE alert_events ADD COLUMN direction INTEGER DEFAULT 0;");
    for (k, v) in DEFAULT_CONFIG {
        db.config_set_if_missing(k, v)?;
    }
    Ok(())
}
