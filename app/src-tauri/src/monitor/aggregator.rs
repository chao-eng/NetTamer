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

/// Sort dimensions for [`Aggregator::top_n`].
#[derive(Debug, Clone, Copy)]
pub enum SortField {
    Upload,
    Download,
    Name,
    Pid,
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Rolls up network events into smoothed per-process rates.
pub struct Aggregator {
    processes: RwLock<HashMap<u32, ProcessAccumulator>>,
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

        for acc in procs.values_mut() {
            let elapsed = now.duration_since(acc.window_start).as_secs_f64().max(1e-6);
            let inst_up = acc.window_bytes_up as f64 / elapsed;
            let inst_down = acc.window_bytes_down as f64 / elapsed;

            acc.last_upload_rate = crate::monitor::ewma(acc.last_upload_rate, inst_up, EWMA_N);
            acc.last_download_rate = crate::monitor::ewma(acc.last_download_rate, inst_down, EWMA_N);

            acc.window_bytes_up = 0;
            acc.window_bytes_down = 0;
            acc.window_start = now;

            let info = self.resolver.resolve(acc.pid);
            out.push(ProcessStats {
                pid: acc.pid,
                name: info.name,
                path: info.path,
                icon_b64: info.icon_b64,
                upload_rate: acc.last_upload_rate,
                download_rate: acc.last_download_rate,
                total_upload: acc.total_upload,
                total_download: acc.total_download,
            });
        }
        out
    }

    /// Return the top `n` processes sorted by `field` / `order`.
    pub fn top_n(&self, n: usize, field: SortField, order: SortOrder) -> Vec<ProcessStats> {
        let mut all = self.snapshot();
        let cmp = |a: &ProcessStats, b: &ProcessStats| -> std::cmp::Ordering {
            let ord = match field {
                SortField::Upload => a.upload_rate.partial_cmp(&b.upload_rate).unwrap_or(std::cmp::Ordering::Equal),
                SortField::Download => a.download_rate.partial_cmp(&b.download_rate).unwrap_or(std::cmp::Ordering::Equal),
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
