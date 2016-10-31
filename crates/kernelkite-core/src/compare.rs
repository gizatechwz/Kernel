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
