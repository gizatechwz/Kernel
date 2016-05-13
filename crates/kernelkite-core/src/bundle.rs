//! Reading and writing profile bundles as deterministic JSON.

use crate::error::{Error, Result};
use crate::model::{ProfileBundle, BUNDLE_SCHEMA_VERSION};

/// Serialize a bundle to pretty JSON. Field order is stable (struct order +
