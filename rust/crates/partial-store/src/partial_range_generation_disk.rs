use anyhow::{Context, Result};
use ghostr_engine::representation::SourceGeneration;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct StoredGeneration {
    pub representation: String,
    pub source: String,
    pub generation: SourceGeneration,
}

pub async fn load(path: &Path) -> Result<Option<StoredGeneration>> {
    let text = match tokio::fs::read_to_string(path).await {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read sparse generation"),
    };
    serde_json::from_str(&text)
        .map(Some)
        .context("parse sparse generation")
}

pub async fn save(path: &Path, stored: &StoredGeneration) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let staging = path.with_extension("json.tmp");
    let json = serde_json::to_vec(stored).context("encode sparse generation")?;
    tokio::fs::write(&staging, json).await?;
    tokio::fs::rename(staging, path)
        .await
        .context("commit sparse generation")
}
