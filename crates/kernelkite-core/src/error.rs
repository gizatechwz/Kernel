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
