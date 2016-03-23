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
