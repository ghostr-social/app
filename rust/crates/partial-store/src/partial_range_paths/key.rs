use anyhow::{bail, Result};

/// Keys become file names, so they may not carry separators or dots.
pub fn validate_key(key: &str) -> Result<()> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_');
    if !key.is_empty() && key.chars().all(allowed) {
        return Ok(());
    }
    bail!("partial store keys must be alphanumeric with dashes or underscores")
}
