use crate::partial_range_paths::validate_key;
use anyhow::{Context as _, Result};
use std::collections::BTreeSet;
use std::path::Path;

/// Every key the root holds. File names are `{key}.{extension}` and a
/// key may carry no dot, so the key is the leading segment.
pub(crate) async fn stored_keys(root: &Path) -> Result<BTreeSet<String>> {
    let mut listing = match tokio::fs::read_dir(root).await {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(error) => return Err(error).context("enumerate video store"),
    };
    let mut keys = BTreeSet::new();
    while let Some(item) = listing
        .next_entry()
        .await
        .context("continue video store enumeration")?
    {
        let name = item.file_name();
        let key = name.to_str().and_then(|name| name.split('.').next());
        if let Some(key) = key.filter(|key| validate_key(key).is_ok()) {
            keys.insert(key.to_owned());
        }
    }
    Ok(keys)
}
