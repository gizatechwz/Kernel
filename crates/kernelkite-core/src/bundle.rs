//! Reading and writing profile bundles as deterministic JSON.

use crate::error::{Error, Result};
use crate::model::{ProfileBundle, BUNDLE_SCHEMA_VERSION};

/// Serialize a bundle to pretty JSON. Field order is stable (struct order +
/// `BTreeMap`), so the same in-memory bundle always yields identical bytes.
pub fn to_json(bundle: &ProfileBundle) -> Result<String> {
    Ok(serde_json::to_string_pretty(bundle)?)
}

/// Parse a bundle from JSON, validating the schema version.
pub fn from_json(raw: &str) -> Result<ProfileBundle> {
    let bundle: ProfileBundle = serde_json::from_str(raw)?;
    if bundle.meta.schema_version != BUNDLE_SCHEMA_VERSION {
        return Err(Error::SchemaMismatch {
            found: bundle.meta.schema_version,
            expected: BUNDLE_SCHEMA_VERSION,
        });
    }
    Ok(bundle)
}

/// Write a bundle to a file path.
pub fn save(bundle: &ProfileBundle, path: impl AsRef<std::path::Path>) -> Result<()> {
    std::fs::write(path, to_json(bundle)?)?;
    Ok(())
}

/// Load a bundle from a file path.
pub fn load(path: impl AsRef<std::path::Path>) -> Result<ProfileBundle> {
    let raw = std::fs::read_to_string(path)?;
    from_json(&raw)
}
