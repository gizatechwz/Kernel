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

    /// Parse a fixture from a JSON string, validating frame ordering.
    pub fn from_json(raw: &str) -> Result<Self> {
        let fx: Fixture = serde_json::from_str(raw)?;
        let mut last = None;
        for f in &fx.frames {
            if let Some(prev) = last {
                if f.t_ms < prev {
                    return Err(Error::Invalid(format!(
                        "fixture frames must be non-decreasing in t_ms (saw {} after {})",
                        f.t_ms, prev
                    )));
                }
            }
            last = Some(f.t_ms);
        }
        Ok(fx)
    }
}

/// Replays a [`Fixture`] frame by frame.
pub struct FixtureSampler {
    frames: std::vec::IntoIter<Frame>,
    host: HostInfo,
}

impl FixtureSampler {
    pub fn new(fixture: Fixture) -> Self {
        FixtureSampler {
            host: HostInfo {
                clock_ticks_per_sec: fixture.clock_ticks_per_sec,
                page_size_bytes: fixture.page_size_bytes,
            },
            frames: fixture.frames.into_iter(),
        }
    }
}

impl Sampler for FixtureSampler {
    fn backend(&self) -> Backend {
        Backend::Fixture
    }

    fn host_info(&self) -> HostInfo {
        self.host
    }

    /// The `_t_ms` hint is ignored: a fixture replays its own recorded
    /// timestamps so the result is byte-for-byte reproducible.
    fn sample(&mut self, _t_ms: u64) -> Result<Option<Frame>> {
        Ok(self.frames.next())
    }
}
