//! Error type shared across the crate. Deliberately dependency-free (no
//! `thiserror`) so the core has a minimal, auditable dependency surface.

use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// An underlying I/O failure (file read, process spawn, ...).
    Io(std::io::Error),
    /// JSON (de)serialization failure.
    Json(serde_json::Error),
    /// A `/proc` entry could not be parsed into the expected shape.
    Parse { path: String, reason: String },
    /// A requested backend exists in the type system but is not implemented
    /// on this build/platform (e.g. the eBPF backend).
    Unsupported(String),
    /// The bundle on disk uses a schema version this build cannot read.
