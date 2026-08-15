//! Aggregates raw [`NetworkEvent`]s into per-process upload/download rates.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::etw::NetworkEvent;
use crate::models::ProcessStats;
use crate::process::Resolver;

/// Number of window samples used for EWMA smoothing (architecture §5.2.2).
const EWMA_N: usize = 5;

/// Per-process running counters.
pub struct ProcessAccumulator {
    pub pid: u32,
    pub total_upload: u64,
    pub total_download: u64,
    pub last_upload_rate: f64,
    pub last_download_rate: f64,
    /// Bytes seen in the current sampling window (reset each snapshot).
    window_bytes_up: u64,
    window_bytes_down: u64,
    window_start: Instant,
}

impl ProcessAccumulator {
    fn new(pid: u32) -> Self {
        ProcessAccumulator {
            pid,
            total_upload: 0,
            total_download: 0,
            last_upload_rate: 0.0,
            last_download_rate: 0.0,
            window_bytes_up: 0,
            window_bytes_down: 0,
            window_start: Instant::now(),
        }
    }
}

/// Sort criteria for the process table (matches UI columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortField {
    UploadRate,
    DownloadRate,
    TotalUpload,
    TotalDownload,
    Name,
    Pid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Rolls up network events into smoothed per-process rates.
pub struct Aggregator {
    processes: RwLock<HashMap<u32, ProcessAccumulator>>,
    #[allow(dead_code)]
    window: Duration,
    resolver: Arc<Resolver>,
}

impl Aggregator {
    pub fn new(window: Duration, resolver: Arc<Resolver>) -> Self {
        Aggregator {
            processes: RwLock::new(HashMap::new()),
            window,
            resolver,
        }
    }

    /// Ingest a single decoded ETW network event.
    pub fn ingest(&self, ev: NetworkEvent) {
        if ev.pid == 0 || ev.pid == std::process::id() {
            return;
        }

        let mut procs = self.processes.write().unwrap();
        let acc = procs.entry(ev.pid).or_insert_with(|| ProcessAccumulator::new(ev.pid));
        match ev.direction {
            crate::models::Direction::Send => {
                acc.total_upload += ev.size as u64;
                acc.window_bytes_up += ev.size as u64;
            }
            crate::models::Direction::Recv => {
                acc.total_download += ev.size as u64;
                acc.window_bytes_down += ev.size as u64;
            }
        }
    }

    /// Compute the current rate snapshot for every tracked process.
    ///
    /// Rates are EWMA-smoothed and the sliding window counters are reset. This
    /// is intended to be called once per `refresh_interval`.
    pub fn snapshot(&self) -> Vec<ProcessStats> {
        let mut procs = self.processes.write().unwrap();
        let now = Instant::now();
        let mut out = Vec::with_capacity(procs.len());

        let self_pid = std::process::id();
        let self_exe_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_lowercase()));

        for acc in procs.values_mut() {
            let elapsed = now.duration_since(acc.window_start).as_secs_f64().max(1e-6);
            let inst_up = acc.window_bytes_up as f64 / elapsed;
            let inst_down = acc.window_bytes_down as f64 / elapsed;

            acc.last_upload_rate = crate::monitor::ewma(acc.last_upload_rate, inst_up, EWMA_N);
            acc.last_download_rate = crate::monitor::ewma(acc.last_download_rate, inst_down, EWMA_N);

            acc.window_bytes_up = 0;
            acc.window_bytes_down = 0;
            acc.window_start = now;

            if acc.pid == 0 || acc.pid == self_pid {
                continue;
            }

            let info = self.resolver.resolve(acc.pid);
            let lower_name = info.name.to_lowercase();
            if lower_name == "system idle process"
                || lower_name == "nettamer.exe"
                || lower_name == "nettamer"
                || lower_name == "msedgewebview2.exe"
                || lower_name == "msedgewebview2"
            {
                continue;
            }

            if let Some(ref self_path) = self_exe_path {
                if !info.path.is_empty() && info.path.to_lowercase() == *self_path {
                    continue;
                }
            }

            out.push(ProcessStats {
                pid: acc.pid,
                name: info.name,
                path: info.path,
                icon_b64: info.icon_b64,
                category: info.category,
                upload_rate: acc.last_upload_rate,
                download_rate: acc.last_download_rate,
                total_upload: acc.total_upload,
                total_download: acc.total_download,
            });
        }
        out
    }

    /// Return the top `n` processes sorted by the specified field.
    #[allow(dead_code)]
    pub fn top_n(&self, n: usize, field: SortField, order: SortOrder) -> Vec<ProcessStats> {
        let mut all = self.snapshot();
        let cmp = |a: &ProcessStats, b: &ProcessStats| -> std::cmp::Ordering {
            let ord = match field {
                SortField::UploadRate => a.upload_rate.partial_cmp(&b.upload_rate).unwrap_or(std::cmp::Ordering::Equal),
                SortField::DownloadRate => a.download_rate.partial_cmp(&b.download_rate).unwrap_or(std::cmp::Ordering::Equal),
                SortField::TotalUpload => a.total_upload.cmp(&b.total_upload),
                SortField::TotalDownload => a.total_download.cmp(&b.total_download),
                SortField::Name => a.name.cmp(&b.name),
                SortField::Pid => a.pid.cmp(&b.pid),
            };
            match order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            }
        };
        all.sort_by(cmp);
        all.truncate(n);
        all
    }
}
