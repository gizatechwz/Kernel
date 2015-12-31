//! Fixture replay sampler (IMPLEMENTED, cross-platform, deterministic).
//!
//! A fixture is simply a JSON array of frames plus the host constants used when
//! it was recorded. Replaying a fixture yields exactly the recorded frames in
//! order, which makes the profiler testable and reproducible on any OS —
//! including Windows, where `/proc` does not exist.

use crate::error::{Error, Result};
