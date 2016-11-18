//! eBPF backend — **NOT IMPLEMENTED** (reserved for the future).
//!
//! kernelkite's roadmap includes an eBPF backend that would attach to
//! scheduler and block-I/O tracepoints for lower-overhead, per-syscall
//! attribution (including per-process network accounting, which the `/proc`
//! backend cannot provide). That work does not exist yet.
//!
//! To guarantee kernelkite never fabricates eBPF data, this stub returns
//! [`Error::Unsupported`] from every call. It is only compiled when the
//! `ebpf` cargo feature is enabled, and even then it does no kernel work.

use crate::error::{Error, Result};
