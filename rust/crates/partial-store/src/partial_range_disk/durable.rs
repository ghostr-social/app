use anyhow::{Context, Result};
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub(super) async fn replace(staging: &Path, target: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target
        .parent()
        .context("partial store path has no parent")?;
    tokio::fs::create_dir_all(parent)
        .await
        .context("create partial store directory")?;
    if let Err(error) = write_synced(staging, bytes).await {
        let _ = super::remove_if_present(staging).await;
        return Err(error);
    }
    if let Err(error) = tokio::fs::rename(staging, target).await {
        let _ = super::remove_if_present(staging).await;
        return Err(error).context("commit partial range manifest");
    }
    sync_directory(parent).await
}

async fn write_synced(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = tokio::fs::File::create(path)
        .await
        .context("write partial range manifest")?;
    file.write_all(bytes)
        .await
        .context("write partial range manifest")?;
    file.sync_all().await.context("sync partial range manifest")
}

#[cfg(unix)]
pub(super) async fn sync_directory(path: &Path) -> Result<()> {
    tokio::fs::File::open(path)
        .await
        .context("open partial store directory")?
        .sync_all()
        .await
        .context("sync partial store directory")
}

#[cfg(not(unix))]
pub(super) async fn sync_directory(_path: &Path) -> Result<()> {
    Ok(())
}
