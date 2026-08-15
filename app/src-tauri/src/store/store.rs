//! SQLite-backed store with an r2d2 connection pool.
//!
//! Implements the `AlertStore`, `ThrottleStore` and `ConfigStore` traits over a
//! pooled `rusqlite` connection. All queries are parameterized.

use std::collections::HashMap;
use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter};

use crate::models::{AlertEvent, AlertHistoryFilter, Error, Policy, Rule};

type Conn = r2d2::PooledConnection<SqliteConnectionManager>;

/// Persistence contract for alert rules + events.
pub trait AlertStore {
    fn save_rule(&self, rule: &Rule) -> Result<(), Error>;
    fn delete_rule(&self, id: &str) -> Result<(), Error>;
    fn list_rules(&self) -> Result<Vec<Rule>, Error>;
    fn save_alert_event(&self, ev: &AlertEvent) -> Result<(), Error>;
    fn list_alert_events(&self, f: &AlertHistoryFilter) -> Result<Vec<AlertEvent>, Error>;
}

/// Persistence contract for throttle policies.
pub trait ThrottleStore {
    fn save_policy(&self, p: &Policy) -> Result<(), Error>;
    fn delete_policy(&self, id: &str) -> Result<(), Error>;
    fn list_policies(&self) -> Result<Vec<Policy>, Error>;
}

/// Persistence contract for the KV config table.
pub trait ConfigStore {
    fn config_get(&self, key: &str) -> Result<Option<String>, Error>;
    fn config_set(&self, key: &str, value: &str) -> Result<(), Error>;
}

/// Pooled SQLite database handle.
pub struct Db {
    pool: Pool<SqliteConnectionManager>,
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Db {
            pool: self.pool.clone(),
        }
    }
}

impl Db {
    /// Open (creating if needed) a SQLite database at `path` and run migrations.
    pub fn new(path: &Path) -> Result<Db, Error> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(8)
            .build(manager)
            .map_err(|e| Error(e.to_string()))?;
        Ok(Db { pool })
    }

    fn conn(&self) -> Result<Conn, Error> {
        self.pool.get().map_err(|e| Error(e.to_string()))
    }

    /// Run an arbitrary batch (used by migrations).
    pub fn execute_batch(&self, sql: &str) -> Result<(), Error> {
        self.conn()?.execute_batch(sql).map_err(|e| Error(e.to_string()))
    }

    // ---------------- Config helpers ----------------

    pub fn config_set_if_missing(&self, key: &str, value: &str) -> Result<(), Error> {
        self.conn()?
            .execute(
                "INSERT OR IGNORE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }

    pub fn config_all(&self) -> Result<HashMap<String, String>, Error> {
        let c = self.conn()?;
        let mut stmt = c
            .prepare("SELECT key, value FROM config")
            .map_err(|e| Error(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| Error(e.to_string()))?;
        let mut out = HashMap::new();
        for row in rows {
            let (k, v) = row.map_err(|e| Error(e.to_string()))?;
            out.insert(k, v);
        }
        Ok(out)
    }
}

// ------------------------- AlertStore -------------------------

impl AlertStore for Db {
    fn save_rule(&self, rule: &Rule) -> Result<(), Error> {
        self.conn()?
            .execute(
                "INSERT OR REPLACE INTO alert_rules \
                 (id, name, process_name, threshold, direction, cooldown_sec, enabled, created_at) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    rule.id,
                    rule.name,
                    rule.process_name,
                    rule.threshold,
                    rule.direction,
                    rule.cooldown_sec,
                    rule.enabled as i64,
                    rule.created_at,
                ],
            )
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }

    fn delete_rule(&self, id: &str) -> Result<(), Error> {
        self.conn()?
            .execute("DELETE FROM alert_rules WHERE id = ?1", params![id])
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }

    fn list_rules(&self) -> Result<Vec<Rule>, Error> {
        let c = self.conn()?;
        let mut stmt = c
            .prepare(
                "SELECT id, name, process_name, threshold, direction, cooldown_sec, enabled, created_at \
                 FROM alert_rules",
            )
            .map_err(|e| Error(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Rule {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    process_name: r.get(2)?,
                    threshold: r.get(3)?,
                    direction: r.get(4)?,
                    cooldown_sec: r.get(5)?,
                    enabled: r.get::<_, i64>(6)? != 0,
                    created_at: r.get(7)?,
                })
            })
            .map_err(|e| Error(e.to_string()))?;
        collect(rows)
    }

    fn save_alert_event(&self, ev: &AlertEvent) -> Result<(), Error> {
        self.conn()?
            .execute(
                "INSERT INTO alert_events \
                 (id, rule_id, process_name, pid, direction, current_rate, threshold, triggered_at) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    ev.id,
                    ev.rule_id,
                    ev.process_name,
                    ev.pid,
                    ev.direction,
                    ev.current_rate,
                    ev.threshold,
                    ev.triggered_at,
                ],
            )
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }

    fn list_alert_events(&self, f: &AlertHistoryFilter) -> Result<Vec<AlertEvent>, Error> {
        let mut sql = String::from(
            "SELECT id, rule_id, process_name, pid, direction, current_rate, threshold, triggered_at \
             FROM alert_events WHERE 1=1",
        );
        let mut args: Vec<String> = Vec::new();
        if let Some(rid) = &f.rule_id {
            sql.push_str(" AND rule_id = ?");
            args.push(rid.clone());
        }
        if let Some(since) = f.since {
            sql.push_str(" AND triggered_at >= ?");
            args.push(since.to_string());
        }
        sql.push_str(" ORDER BY triggered_at DESC");
        if let Some(lim) = f.limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }

        let c = self.conn()?;
        let mut stmt = c.prepare(&sql).map_err(|e| Error(e.to_string()))?;
        let rows = stmt
            .query_map(params_from_iter(args.iter()), |r| {
                Ok(AlertEvent {
                    id: r.get(0)?,
                    rule_id: r.get(1)?,
                    process_name: r.get(2)?,
                    pid: r.get(3)?,
                    direction: r.get(4).unwrap_or(0),
                    current_rate: r.get(5)?,
                    threshold: r.get(6)?,
                    triggered_at: r.get(7)?,
                })
            })
            .map_err(|e| Error(e.to_string()))?;
        collect(rows)
    }
}

// ------------------------- ThrottleStore -------------------------

impl ThrottleStore for Db {
    fn save_policy(&self, p: &Policy) -> Result<(), Error> {
        self.conn()?
            .execute(
                "INSERT OR REPLACE INTO throttle_policies \
                 (id, name, process_name, rate_limit_bps, limit_upload, limit_download, active, created_at) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    p.id,
                    p.name,
                    p.process_name,
                    p.rate_limit_bps,
                    p.limit_upload as i64,
                    p.limit_download as i64,
                    p.active as i64,
                    p.created_at,
                ],
            )
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }

    fn delete_policy(&self, id: &str) -> Result<(), Error> {
        self.conn()?
            .execute("DELETE FROM throttle_policies WHERE id = ?1", params![id])
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }

    fn list_policies(&self) -> Result<Vec<Policy>, Error> {
        let c = self.conn()?;
        let mut stmt = c
            .prepare(
                "SELECT id, name, process_name, rate_limit_bps, limit_upload, limit_download, active, created_at \
                 FROM throttle_policies",
            )
            .map_err(|e| Error(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Policy {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    process_name: r.get(2)?,
                    rate_limit_bps: r.get(3)?,
                    limit_upload: r.get::<_, i64>(4)? != 0,
                    limit_download: r.get::<_, i64>(5)? != 0,
                    active: r.get::<_, i64>(6)? != 0,
                    created_at: r.get(7)?,
                })
            })
            .map_err(|e| Error(e.to_string()))?;
        collect(rows)
    }
}

// ------------------------- ConfigStore -------------------------

impl ConfigStore for Db {
    fn config_get(&self, key: &str) -> Result<Option<String>, Error> {
        let c = self.conn()?;
        let mut stmt = c
            .prepare("SELECT value FROM config WHERE key = ?1")
            .map_err(|e| Error(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![key], |r| r.get::<_, String>(0))
            .map_err(|e| Error(e.to_string()))?;
        match rows.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(Error(e.to_string())),
            None => Ok(None),
        }
    }

    fn config_set(&self, key: &str, value: &str) -> Result<(), Error> {
        self.conn()?
            .execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| Error(e.to_string()))?;
        Ok(())
    }
}

/// Helper: collect a `rusqlite::MappedRows` iterator into a `Vec`, mapping
/// rusqlite errors into our [`Error`] type.
fn collect<T>(rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>) -> Result<Vec<T>, Error> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| Error(e.to_string()))?);
    }
    Ok(out)
}
