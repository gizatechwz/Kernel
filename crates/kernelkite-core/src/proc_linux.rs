//! Linux `/proc` sampler (IMPLEMENTED).
//!
//! This backend reads the procfs pseudo-filesystem to build per-process
//! samples: CPU time from `/proc/<pid>/stat`, memory from `/proc/<pid>/statm`,
//! block I/O from `/proc/<pid>/io`, open file descriptors from
//! `/proc/<pid>/fd`, and an optional host-wide network summary from
//! `/proc/net/dev`.
//!
//! The whole module is compiled only on Linux. On other platforms the public
//! type is provided by a small `cfg`-guarded shim (see the bottom of this
//! file) so the crate compiles everywhere while the real logic stays honest
//! about what it can observe.

use crate::error::{Error, Result};
use crate::model::{Backend, Frame, NetworkSummary, ProcessSample};
use crate::sampler::{HostInfo, Sampler};

/// Optional filter: which pids to include in each frame.
#[derive(Debug, Clone)]
pub enum PidFilter {
    /// Every process visible to the caller.
    All,
    /// Only the given pid and its descendants (recomputed each frame).
    Subtree(i32),
    /// Only this explicit set of pids.
    Set(Vec<i32>),
}

/// Reads `/proc` on each `sample` call.
pub struct ProcSampler {
    filter: PidFilter,
    with_network: bool,
    host: HostInfo,
    root: std::path::PathBuf,
}

impl ProcSampler {
    /// Create a sampler rooted at the real `/proc`.
    pub fn new(filter: PidFilter, with_network: bool) -> Self {
        Self::rooted(filter, with_network, "/proc")
    }

    /// Create a sampler rooted at an arbitrary directory. Used by tests to
    /// point the parser at a synthetic procfs tree.
    pub fn rooted(
        filter: PidFilter,
        with_network: bool,
        root: impl Into<std::path::PathBuf>,
    ) -> Self {
        ProcSampler {
            filter,
            with_network,
            host: detect_host_info(),
            root: root.into(),
        }
    }
