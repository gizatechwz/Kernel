//! Error type shared across the crate. Deliberately dependency-free (no
//! `thiserror`) so the core has a minimal, auditable dependency surface.

use std::fmt;

#[derive(Debug)]
