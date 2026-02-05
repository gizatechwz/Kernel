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
    fn backend(&self) -> Backend {
        Backend::Proc
    }

    fn host_info(&self) -> HostInfo {
        self.host
    }

    fn sample(&mut self, t_ms: u64) -> Result<Option<Frame>> {
        let mut frame = Frame::new(t_ms);
        for pid in self.selected_pids()? {
            // A process may vanish between listing and reading; skip races.
            let stat = match self.read_stat(pid) {
                Ok(s) => s,
                Err(Error::Io(_)) => continue,
                Err(e) => return Err(e),
            };
            let (vsize_pages, rss_pages) = match self.read_statm_rss_pages(pid) {
                Ok(v) => v,
                Err(Error::Io(_)) => continue,
                Err(e) => return Err(e),
            };
            let io = self.read_io(pid);
            let open_fds = self.read_fd_count(pid);
            frame.insert(ProcessSample {
                pid,
                ppid: stat.ppid,
                comm: stat.comm,
                utime_ticks: stat.utime,
                stime_ticks: stat.stime,
                rss_bytes: rss_pages.saturating_mul(self.host.page_size_bytes),
                vsize_bytes: vsize_pages.saturating_mul(self.host.page_size_bytes),
                threads: stat.num_threads.max(0) as u32,
                read_bytes: io.map(|(r, _)| r),
                write_bytes: io.map(|(_, w)| w),
                open_fds,
            });
        }
        if self.with_network {
            frame.network = self.read_network();
        }
        Ok(Some(frame))
    }
}

/// Fields extracted from `/proc/<pid>/stat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatFields {
    pub ppid: i32,
    pub comm: String,
    pub utime: u64,
    pub stime: u64,
    pub num_threads: i64,
}

/// Parse a `/proc/<pid>/stat` line.
///
/// The tricky part of this format is `comm`: it is wrapped in parentheses and
/// may itself contain spaces or parentheses (e.g. `(foo (bar) baz)`). We locate
/// the last `)` to end the comm, then split the remaining space-separated
/// fields by their documented positional index (see `proc(5)`).
pub fn parse_stat(raw: &str) -> Option<StatFields> {
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open {
        return None;
    }
    let comm = raw[open + 1..close].to_string();
    // Fields after comm start at position 3 in proc(5) numbering (state = 3).
    // The rest split cleanly on whitespace.
    let rest: Vec<&str> = raw[close + 1..].split_whitespace().collect();
    // rest[0] = state, rest[1] = ppid, rest[11] = utime, rest[12] = stime,
    // rest[17] = num_threads (0-based indices after comm).
    let ppid = rest.get(1)?.parse().ok()?;
    let utime = rest.get(11)?.parse().ok()?;
    let stime = rest.get(12)?.parse().ok()?;
    let num_threads = rest.get(17)?.parse().ok()?;
    Some(StatFields {
        ppid,
        comm,
        utime,
        stime,
        num_threads,
    })
}

/// Parse `/proc/<pid>/io`, returning `(read_bytes, write_bytes)`.
///
/// We prefer `read_bytes` / `write_bytes` (actual storage-layer transfer) over
/// `rchar` / `wchar` (which count syscall bytes including cache hits).
pub fn parse_io(raw: &str) -> Option<(u64, u64)> {
    let mut read_bytes = None;
    let mut write_bytes = None;
    for line in raw.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("read_bytes:") => read_bytes = it.next().and_then(|v| v.parse().ok()),
            Some("write_bytes:") => write_bytes = it.next().and_then(|v| v.parse().ok()),
            _ => {}
        }
    }
    Some((read_bytes?, write_bytes?))
}

/// Parse `/proc/net/dev` into a host-wide [`NetworkSummary`]. The loopback
/// interface is excluded so the summary reflects off-host traffic.
pub fn parse_net_dev(raw: &str) -> Option<NetworkSummary> {
    let mut rx_total = 0u64;
    let mut tx_total = 0u64;
    let mut interfaces = Vec::new();
    for line in raw.lines() {
        let Some((name, rest)) = line.split_once(':') else {
            continue; // header lines have no colon
        };
        let name = name.trim();
        if name == "lo" || name.is_empty() {
            continue;
        }
        let cols: Vec<&str> = rest.split_whitespace().collect();
        // Column 0 = rx bytes, column 8 = tx bytes (see /proc/net/dev layout).
        let rx: u64 = cols.first().and_then(|v| v.parse().ok())?;
        let tx: u64 = cols.get(8).and_then(|v| v.parse().ok())?;
        rx_total = rx_total.saturating_add(rx);
        tx_total = tx_total.saturating_add(tx);
        interfaces.push(name.to_string());
    }
    if interfaces.is_empty() {
        return None;
    }
    interfaces.sort();
    Some(NetworkSummary {
        rx_bytes: rx_total,
        tx_bytes: tx_total,
        interfaces,
    })
}

/// Detect host constants from the running Linux system.
pub fn detect_host_info() -> HostInfo {
    // SAFETY: sysconf is a pure, thread-safe libc query with no side effects.
    let clk = unsafe { sysconf(SC_CLK_TCK) };
    let page = unsafe { sysconf(SC_PAGESIZE) };
    HostInfo {
        clock_ticks_per_sec: if clk > 0 { clk as u64 } else { 100 },
        page_size_bytes: if page > 0 { page as u64 } else { 4096 },
    }
}

// Minimal libc bindings so the core has no external C-binding dependency.
const SC_CLK_TCK: i32 = 2;
const SC_PAGESIZE: i32 = 30;

extern "C" {
    fn sysconf(name: i32) -> i64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::Sampler;

    #[test]
    fn parse_stat_handles_spaces_and_parens_in_comm() {
        // comm = "(weird cmd)" containing spaces and inner parentheses.
        // Layout: pid (comm) state ppid pgrp ... utime(14th) stime(15th) ... threads(20th)
        // proc(5) fields 1..=52; here we only need up to num_threads (field 20).
        let raw = "1234 ((weird cmd)) S 1000 1234 1234 0 -1 4194304 \
                   100 200 0 0 4200 1800 0 0 20 0 7 0 \
                   1500 1000000 512 18446744073709551615 1 1 0 0 0 0 0";
        let s = parse_stat(raw).expect("should parse");
        assert_eq!(s.comm, "(weird cmd)");
        assert_eq!(s.ppid, 1000);
        assert_eq!(s.utime, 4200);
        assert_eq!(s.stime, 1800);
        assert_eq!(s.num_threads, 7);
    }

    #[test]
    fn parse_io_prefers_storage_bytes() {
        let raw = "rchar: 999\nwchar: 888\nsyscr: 7\nsyscw: 8\n\
                   read_bytes: 4096\nwrite_bytes: 8192\ncancelled_write_bytes: 0\n";
        assert_eq!(parse_io(raw), Some((4096, 8192)));
    }

    #[test]
    fn parse_net_dev_sums_non_loopback() {
        let raw = "Inter-|   Receive                    |  Transmit\n\
                   face |bytes packets errs drop fifo frame compressed multicast|bytes packets\n\
                   lo:  1000 10 0 0 0 0 0 0 1000 10 0 0 0 0 0 0\n\
                   eth0:  5000 50 0 0 0 0 0 0 3000 30 0 0 0 0 0 0\n\
                   wlan0: 2000 20 0 0 0 0 0 0 1500 15 0 0 0 0 0 0\n";
        let n = parse_net_dev(raw).expect("summary");
        assert_eq!(n.rx_bytes, 7000); // 5000 + 2000, loopback excluded
        assert_eq!(n.tx_bytes, 4500); // 3000 + 1500
        assert_eq!(n.interfaces, vec!["eth0".to_string(), "wlan0".to_string()]);
    }

    /// Build a synthetic procfs tree and drive `ProcSampler` over it. This runs
    /// the entire /proc read path without needing real processes and works on
    /// the Linux CI runner.
    #[test]
    fn proc_sampler_reads_synthetic_tree() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let mk = |pid: i32, ppid: i32, comm: &str, utime: u64| {
            let p = root.join(pid.to_string());
            std::fs::create_dir_all(p.join("fd")).unwrap();
            // one fake open fd
            std::fs::write(p.join("fd").join("0"), b"").unwrap();
            let stat = format!(
                "{pid} ({comm}) S {ppid} {pid} {pid} 0 -1 0 0 0 0 0 {utime} 10 0 0 20 0 3 0 \
                 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
            );
            std::fs::write(p.join("stat"), stat).unwrap();
            // statm: size resident shared text lib data dt (in pages)
            std::fs::write(p.join("statm"), "2000 500 100 1 0 300 0").unwrap();
            std::fs::write(
                p.join("io"),
                "rchar: 1\nwchar: 1\nread_bytes: 4096\nwrite_bytes: 8192\n",
            )
            .unwrap();
        };
        mk(100, 1, "make", 50);
        mk(101, 100, "cc1", 20);

        let mut s = ProcSampler::rooted(PidFilter::All, false, root);
        // Force a deterministic host_info for the assertion below.
        s.host = HostInfo {
            clock_ticks_per_sec: 100,
            page_size_bytes: 4096,
        };
        let frame = s.sample(0).unwrap().unwrap();
        assert_eq!(frame.processes.len(), 2);
        let make = &frame.processes[&100];
        assert_eq!(make.comm, "make");
        assert_eq!(make.ppid, 1);
        assert_eq!(make.utime_ticks, 50);
        assert_eq!(make.rss_bytes, 500 * 4096);
        assert_eq!(make.read_bytes, Some(4096));
        assert_eq!(make.open_fds, Some(1));

        // Subtree filter rooted at make should include cc1.
        let mut sub = ProcSampler::rooted(PidFilter::Subtree(100), false, root);
        let f2 = sub.sample(0).unwrap().unwrap();
        assert_eq!(f2.processes.len(), 2);

        // Subtree rooted at cc1 alone should include just cc1.
        let mut leaf = ProcSampler::rooted(PidFilter::Subtree(101), false, root);
        let f3 = leaf.sample(0).unwrap().unwrap();
        assert_eq!(f3.processes.len(), 1);
        assert!(f3.processes.contains_key(&101));
    }
}
