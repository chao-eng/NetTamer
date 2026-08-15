//! Shared data contracts between the Rust backend and the TypeScript frontend.
//!
//! Every struct that crosses the Tauri command / event boundary is declared here
//! and serialized with `#[serde(rename_all = "camelCase")]` so the JSON matches
//! the frontend's camelCase TypeScript interfaces exactly.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single process's network statistics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStats {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub icon_b64: String,
    pub upload_rate: f64,   // bytes/sec
    pub download_rate: f64, // bytes/sec
    pub total_upload: u64,
    pub total_download: u64,
}

/// A user-defined alert rule (threshold on a process's send/recv rate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub process_name: String,
    pub threshold: f64,    // bytes/sec
    pub direction: i32,     // 0 = Upload, 1 = Download, 2 = Both
    pub cooldown_sec: i64,
    pub enabled: bool,
    pub created_at: i64,
}

/// An emitted alert event (a rule fired).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub process_name: String,
    pub pid: u32,
    pub direction: i32,     // 0 = Upload, 1 = Download, 2 = Both
    pub current_rate: f64,
    pub threshold: f64,
    pub triggered_at: i64,
}

/// A firewall block rule applied to a process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRule {
    pub id: String,
    pub name: String,
    pub process_name: String,
    pub active: bool,
    pub created_at: i64,
}

/// Filter for querying the alert history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertHistoryFilter {
    pub rule_id: Option<String>,
    pub since: Option<i64>,
    pub limit: Option<usize>,
}

/// Aggregate system-wide speeds, emitted every second.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub total_upload_rate: f64,
    pub total_download_rate: f64,
}

/// Traffic direction (used internally for traffic filtering and alerts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Send,
    Recv,
}

/// Local error type. We keep it as a single string-wrapping error so every
/// module can `map_err(|e| Error(e.to_string()))` into a `Result<_, String>`
/// that Tauri commands return to the frontend.
#[derive(Debug, Clone)]
pub struct Error(pub String);

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Error(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error(s.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(e.to_string())
    }
}

/// Current unix epoch in seconds (i64).
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// A best-effort unique id (nanosecond-precise timestamp + counter suffix).
pub fn new_id() -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{}", nanos, n)
}
