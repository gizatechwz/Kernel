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
