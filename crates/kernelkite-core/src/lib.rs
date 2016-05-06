//! # kernelkite-core
//!
//! Core engine for **kernelkite**, a Linux-first local developer-loop process
//! profiler. It samples process resource usage over time, assembles
//! deterministic [`ProfileBundle`](model::ProfileBundle)s, and produces
//! before/after comparisons for a build/test loop.
//!
//! ## Backends
//!
//! | Backend  | Status            | Platforms      |
//! |----------|-------------------|----------------|
//! | `proc`   | **implemented**   | Linux only     |
//! | `fixture`| **implemented**   | any OS         |
//! | `ebpf`   | *future, stubbed* | none (feature) |
//!
//! The `/proc` sampler ([`proc_linux::ProcSampler`]) is the real live backend.
//! The [`fixture::FixtureSampler`] replays recorded frames deterministically on
//! any OS (including Windows). The [`ebpf::EbpfSampler`] is a placeholder for a
//! future backend and **always** returns an error — kernelkite never fabricates
//! eBPF data.
//!
//! ## Cross-platform note
//!
//! `/proc` only exists on Linux. On other platforms the [`proc_linux`] module
//! is replaced by a compile-time shim whose `ProcSampler::sample` returns
//! [`error::Error::Unsupported`], so the whole crate builds on Windows and
//! macOS while remaining honest about what it can observe there.

pub mod bundle;
pub mod compare;
pub mod ebpf;
pub mod error;
pub mod fixture;
pub mod model;
pub mod runner;
pub mod sampler;

#[cfg(target_os = "linux")]
pub mod proc_linux;

/// Non-Linux shim for the `/proc` backend so the crate compiles everywhere.
/// The real implementation lives in the Linux-only module of the same name.
#[cfg(not(target_os = "linux"))]
pub mod proc_linux {
    use crate::error::{Error, Result};
    use crate::model::{Backend, Frame};
    use crate::sampler::{HostInfo, Sampler};

    /// Mirrors the Linux filter type so callers compile unchanged.
    #[derive(Debug, Clone)]
    pub enum PidFilter {
        All,
        Subtree(i32),
        Set(Vec<i32>),
    }

    /// Non-Linux stand-in for the `/proc` sampler. Construction succeeds so
    /// code paths type-check, but sampling reports that `/proc` is absent.
    pub struct ProcSampler {
        host: HostInfo,
    }

    impl ProcSampler {
        pub fn new(_filter: PidFilter, _with_network: bool) -> Self {
            ProcSampler {
                host: HostInfo::default(),
            }
        }

        pub fn rooted(
            _filter: PidFilter,
            _with_network: bool,
            _root: impl Into<std::path::PathBuf>,
        ) -> Self {
            ProcSampler {
                host: HostInfo::default(),
            }
        }
    }

    impl Sampler for ProcSampler {
        fn backend(&self) -> Backend {
            Backend::Proc
        }

        fn host_info(&self) -> HostInfo {
            self.host
