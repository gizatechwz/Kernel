//! Capture loop and live command runner.
//!
//! [`capture`] drives any [`Sampler`] on a fixed cadence and assembles a
//! [`ProfileBundle`]. [`run_and_profile`] additionally spawns a real child
//! process, samples while it runs, and records its exit code — the "live
//! developer-loop" mode. On Linux the runner defaults to profiling the child's
//! process subtree so build tools that fork compilers/linkers are captured
//! along with their parent.

use crate::error::Result;
use crate::model::{Backend, CaptureMeta, ProfileBundle, BUNDLE_SCHEMA_VERSION};
use crate::sampler::Sampler;
use std::time::{Duration, Instant};

/// Options controlling a capture session.
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    /// Human label stored in the bundle (e.g. "before").
    pub label: String,
    /// Milliseconds between frames.
    pub interval_ms: u64,
    /// Maximum number of frames to collect (safety bound for `capture`).
    pub max_frames: usize,
    /// Record wall-clock start time in the bundle metadata. Disable for
