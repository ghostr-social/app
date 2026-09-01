use super::MAX_SNAPSHOT_BYTES;
use anyhow::{ensure, Context as _};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

pub(super) async fn read(path: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("open event cache snapshot"),
    };
    let mut bytes = Vec::new();
    file.take(MAX_SNAPSHOT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .context("read event cache snapshot")?;
    ensure!(
        bytes.len() <= MAX_SNAPSHOT_BYTES,
        "event cache snapshot is oversized"
    );
    Ok(Some(bytes))
}

pub(super) async fn replace(path: &Path, body: &[u8]) -> anyhow::Result<()> {
    ensure!(
        body.len() <= MAX_SNAPSHOT_BYTES,
        "event cache snapshot is oversized"
    );
    let parent = parent(path)?;
    tokio::fs::create_dir_all(&parent)
        .await
        .context("create event cache directory")?;
    let stage = stage_path(path);
    write_stage(&stage, body).await?;
    tokio::fs::rename(&stage, path)
        .await
        .context("publish event cache snapshot")?;
    sync_directory(parent).await
}

pub(super) async fn clear(path: &Path) -> anyhow::Result<()> {
    remove_if_present(path).await?;
    remove_if_present(&stage_path(path)).await?;
    Ok(())
}

async fn write_stage(path: &Path, body: &[u8]) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::create(path)
        .await
        .context("stage event cache snapshot")?;
    file.write_all(body)
        .await
        .context("write event cache snapshot")?;
    file.sync_all().await.context("sync event cache snapshot")
}

async fn remove_if_present(path: &Path) -> anyhow::Result<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("remove event cache snapshot"),
    }
}

async fn sync_directory(path: PathBuf) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || std::fs::File::open(path)?.sync_all())
        .await
        .context("join event cache directory sync")?
        .context("sync event cache directory")
}

fn parent(path: &Path) -> anyhow::Result<PathBuf> {
    path.parent()
        .map(Path::to_path_buf)
        .context("event cache snapshot has no parent")
}

fn stage_path(path: &Path) -> PathBuf {
    path.with_extension("stage")
}
