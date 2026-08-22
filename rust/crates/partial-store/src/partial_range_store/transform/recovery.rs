use super::{record, transaction};
use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_representation_disk as identity_disk;
use anyhow::Result;

#[cfg(test)]
#[path = "recovery_test_fixture.rs"]
mod test_fixture;
#[cfg(test)]
#[path = "recovery_test_paths.rs"]
mod test_paths;
#[cfg(test)]
#[path = "recovery_test.rs"]
mod tests;

pub(super) async fn recover(paths: &StorePaths, key: &str) -> Result<()> {
    let transform = paths.transform(key);
    if disk::file_len(&transform.commit()).await?.is_none() {
        return transaction::discard_staging(paths, key).await;
    }
    if committed(paths, key).await.unwrap_or(false) {
        transaction::finish(paths, key).await
    } else {
        transaction::rollback(paths, key).await
    }
}

async fn committed(paths: &StorePaths, key: &str) -> Result<bool> {
    let Some(record) = record::load(paths, key).await? else {
        return Ok(false);
    };
    let identity = identity_disk::load(&paths.representation(key)).await?;
    if identity.as_deref() != Some(record.output()) {
        return Ok(false);
    }
    if disk::file_len(&paths.completed(key)).await? != Some(record.bytes()) {
        return Ok(false);
    }
    let manifest = disk::load_manifest(&paths.manifest(key)).await?;
    if manifest.total_len() != Some(record.bytes()) || !manifest.is_complete() {
        return Ok(false);
    }
    Ok(disk::sha256_file(&paths.completed(key)).await? == record.digest())
}
