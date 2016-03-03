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
    SchemaMismatch { found: u32, expected: u32 },
    /// Generic invalid-input error with a human-readable message.
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::Json(e) => write!(f, "json error: {e}"),
            Error::Parse { path, reason } => {
                write!(f, "failed to parse {path}: {reason}")
            }
            Error::Unsupported(what) => write!(f, "unsupported: {what}"),
            Error::SchemaMismatch { found, expected } => write!(
                f,
                "bundle schema version {found} is not supported (this build expects {expected})"
            ),
            Error::Invalid(m) => write!(f, "invalid input: {m}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
