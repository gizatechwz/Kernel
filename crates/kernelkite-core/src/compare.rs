//! Analysis of a single bundle and before/after comparison of two bundles.
//!
//! Raw bundles store cumulative kernel counters. This module turns them into
//! the human-meaningful aggregates a developer cares about during a build/test
//! loop: total CPU seconds, peak memory, bytes read/written, process count and
//! wall-clock duration — then diffs two such summaries.

use crate::model::ProfileBundle;
use serde::{Deserialize, Serialize};

/// Derived aggregate metrics for one bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub label: String,
    pub backend: String,
    /// Wall-clock span of the timeline in milliseconds.
    pub duration_ms: u64,
    /// Number of frames captured.
    pub frames: usize,
    /// Distinct processes observed over the whole timeline.
    pub distinct_pids: usize,
    /// Total CPU seconds (user + system) consumed across all processes,
    /// measured as the summed increase in CPU ticks from first to last time
    /// each pid was seen, divided by the host tick rate.
    pub cpu_seconds: f64,
    /// Peak total resident memory across processes in any single frame (bytes).
    pub peak_rss_bytes: u64,
    /// Total bytes read (delta of cumulative counters), if I/O was recorded.
    pub read_bytes: Option<u64>,
    /// Total bytes written (delta of cumulative counters), if I/O was recorded.
    pub write_bytes: Option<u64>,
    /// Net off-host bytes received during the timeline, if network was sampled.
    pub net_rx_bytes: Option<u64>,
    /// Net off-host bytes transmitted during the timeline, if sampled.
    pub net_tx_bytes: Option<u64>,
}

/// First and last observation of a pid's cumulative counters.
#[derive(Default, Clone, Copy)]
struct Track {
    first_cpu: Option<u64>,
    last_cpu: u64,
    first_read: Option<u64>,
    last_read: u64,
    first_write: Option<u64>,
    last_write: u64,
    saw_io: bool,
}

/// Compute the derived [`Summary`] for a bundle.
pub fn summarize(bundle: &ProfileBundle) -> Summary {
    let ticks_per_sec = bundle.meta.clock_ticks_per_sec.max(1) as f64;
    let mut tracks: std::collections::BTreeMap<i32, Track> = std::collections::BTreeMap::new();
    let mut peak_rss = 0u64;

    // Network: cumulative counters, so take last - first of the summed totals.
    let mut first_net: Option<(u64, u64)> = None;
    let mut last_net: Option<(u64, u64)> = None;

    for frame in &bundle.frames {
        let mut frame_rss = 0u64;
        for s in frame.processes.values() {
            frame_rss = frame_rss.saturating_add(s.rss_bytes);
            let cpu = s.utime_ticks + s.stime_ticks;
            let t = tracks.entry(s.pid).or_default();
            if t.first_cpu.is_none() {
                t.first_cpu = Some(cpu);
            }
            t.last_cpu = cpu;
            if let (Some(r), Some(w)) = (s.read_bytes, s.write_bytes) {
                t.saw_io = true;
                if t.first_read.is_none() {
                    t.first_read = Some(r);
                }
                t.last_read = r;
                if t.first_write.is_none() {
                    t.first_write = Some(w);
                }
                t.last_write = w;
            }
        }
        peak_rss = peak_rss.max(frame_rss);
        if let Some(n) = &frame.network {
            if first_net.is_none() {
                first_net = Some((n.rx_bytes, n.tx_bytes));
            }
            last_net = Some((n.rx_bytes, n.tx_bytes));
        }
    }

    let mut cpu_ticks = 0u64;
    let mut read_total = 0u64;
    let mut write_total = 0u64;
    let mut any_io = false;
    for t in tracks.values() {
        if let Some(first) = t.first_cpu {
            cpu_ticks = cpu_ticks.saturating_add(t.last_cpu.saturating_sub(first));
        }
        if t.saw_io {
            any_io = true;
            if let Some(fr) = t.first_read {
                read_total = read_total.saturating_add(t.last_read.saturating_sub(fr));
            }
            if let Some(fw) = t.first_write {
                write_total = write_total.saturating_add(t.last_write.saturating_sub(fw));
            }
        }
    }

    let (net_rx, net_tx) = match (first_net, last_net) {
        (Some((rx0, tx0)), Some((rx1, tx1))) => {
            (Some(rx1.saturating_sub(rx0)), Some(tx1.saturating_sub(tx0)))
        }
        _ => (None, None),
    };

    Summary {
        label: bundle.meta.label.clone(),
        backend: bundle.meta.backend.as_str().to_string(),
        duration_ms: bundle.duration_ms(),
        frames: bundle.frames.len(),
        distinct_pids: tracks.len(),
        cpu_seconds: cpu_ticks as f64 / ticks_per_sec,
        peak_rss_bytes: peak_rss,
        read_bytes: if any_io { Some(read_total) } else { None },
        write_bytes: if any_io { Some(write_total) } else { None },
        net_rx_bytes: net_rx,
        net_tx_bytes: net_tx,
    }
}

/// A single before/after metric delta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    pub metric: String,
    pub before: f64,
    pub after: f64,
    pub abs_change: f64,
    /// Percentage change relative to `before`. `None` when `before` is zero.
    pub pct_change: Option<f64>,
}

impl Delta {
    fn new(metric: &str, before: f64, after: f64) -> Self {
        let abs = after - before;
        let pct = if before.abs() > f64::EPSILON {
            Some(abs / before * 100.0)
        } else {
            None
        };
        Delta {
            metric: metric.to_string(),
            before,
            after,
            abs_change: abs,
            pct_change: pct,
        }
    }
}
