//! Fixture replay sampler (IMPLEMENTED, cross-platform, deterministic).
//!
//! A fixture is simply a JSON array of frames plus the host constants used when
//! it was recorded. Replaying a fixture yields exactly the recorded frames in
//! order, which makes the profiler testable and reproducible on any OS —
//! including Windows, where `/proc` does not exist.

use crate::error::{Error, Result};
use crate::model::{Backend, Frame};
use crate::sampler::{HostInfo, Sampler};
use serde::{Deserialize, Serialize};

/// On-disk fixture format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fixture {
    /// Clock ticks per second on the recording host.
    pub clock_ticks_per_sec: u64,
    /// Page size in bytes on the recording host.
    pub page_size_bytes: u64,
    /// Recorded frames, expected to be in ascending `t_ms` order.
    pub frames: Vec<Frame>,
}

impl Fixture {
    /// Load a fixture from a JSON file.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_json(&raw)
    }

