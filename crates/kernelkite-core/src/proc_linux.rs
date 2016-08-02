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

    fn list_pids(&self) -> Result<Vec<i32>> {
        let mut pids = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(pid) = name.parse::<i32>() {
                    pids.push(pid);
                }
            }
        }
        pids.sort_unstable();
        Ok(pids)
    }

    fn selected_pids(&self) -> Result<Vec<i32>> {
        let all = self.list_pids()?;
        match &self.filter {
            PidFilter::All => Ok(all),
            PidFilter::Set(set) => Ok(all.into_iter().filter(|p| set.contains(p)).collect()),
            PidFilter::Subtree(root) => {
                // Build ppid map for all visible pids, then collect descendants.
                let mut ppid_of = std::collections::BTreeMap::new();
                for pid in &all {
                    if let Ok(s) = self.read_stat(*pid) {
                        ppid_of.insert(*pid, s.ppid);
                    }
                }
                let mut keep = std::collections::BTreeSet::new();
                keep.insert(*root);
                // Iterate to a fixed point over the (small) pid set.
                let mut changed = true;
                while changed {
                    changed = false;
                    for (pid, ppid) in &ppid_of {
                        if keep.contains(ppid) && keep.insert(*pid) {
                            changed = true;
                        }
                    }
                }
                Ok(all.into_iter().filter(|p| keep.contains(p)).collect())
            }
        }
    }

    fn read_stat(&self, pid: i32) -> Result<StatFields> {
        let path = self.root.join(pid.to_string()).join("stat");
        let raw = std::fs::read_to_string(&path)?;
        parse_stat(&raw).ok_or_else(|| Error::Parse {
            path: path.display().to_string(),
            reason: "unexpected /proc/<pid>/stat layout".into(),
        })
    }

    fn read_statm_rss_pages(&self, pid: i32) -> Result<(u64, u64)> {
        let path = self.root.join(pid.to_string()).join("statm");
        let raw = std::fs::read_to_string(&path)?;
        let mut it = raw.split_whitespace();
        let vsize_pages: u64 =
            it.next()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| Error::Parse {
                    path: path.display().to_string(),
                    reason: "missing vsize field".into(),
                })?;
        let rss_pages: u64 =
            it.next()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| Error::Parse {
                    path: path.display().to_string(),
                    reason: "missing rss field".into(),
                })?;
        Ok((vsize_pages, rss_pages))
    }

    fn read_io(&self, pid: i32) -> Option<(u64, u64)> {
        let path = self.root.join(pid.to_string()).join("io");
        let raw = std::fs::read_to_string(path).ok()?;
        parse_io(&raw)
    }

    fn read_fd_count(&self, pid: i32) -> Option<u32> {
        let path = self.root.join(pid.to_string()).join("fd");
        let count = std::fs::read_dir(path).ok()?.count();
        Some(count as u32)
    }

    fn read_network(&self) -> Option<NetworkSummary> {
        let path = self.root.join("net").join("dev");
        let raw = std::fs::read_to_string(path).ok()?;
        parse_net_dev(&raw)
    }
}

impl Sampler for ProcSampler {
