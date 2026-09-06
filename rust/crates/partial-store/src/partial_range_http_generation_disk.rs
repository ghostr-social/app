use crate::partial_range_disk as disk;
use anyhow::{Context as _, Result};
use ghostr_engine::representation::HttpGenerationKey;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
#[path = "partial_range_http_generation_disk/selection_test.rs"]
mod selection_test;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StoredHttpGeneration {
    pub representation: String,
    pub source: String,
    pub key: HttpGenerationKey,
}

pub enum StoredHttpGenerationLoad {
    Missing,
    Invalid,
    Valid(StoredHttpGeneration),
}

pub async fn load(path: &Path) -> Result<StoredHttpGenerationLoad> {
    let text = match tokio::fs::read_to_string(path).await {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StoredHttpGenerationLoad::Missing);
        }
        Err(error) => return Err(error).context("read HTTP generation"),
    };
    let stored: StoredHttpGeneration = match serde_json::from_str(&text) {
        Ok(stored) => stored,
        Err(_) => return Ok(StoredHttpGenerationLoad::Invalid),
    };
    let key = HttpGenerationKey::try_new(stored.key.final_url(), stored.key.validator().cloned())
        .ok()
        .map(|key| key.with_request_selection(stored.key.request_selection()));
    Ok(match key {
        Some(key) => StoredHttpGenerationLoad::Valid(StoredHttpGeneration { key, ..stored }),
        None => StoredHttpGenerationLoad::Invalid,
    })
}

pub async fn save(path: &Path, stored: &StoredHttpGeneration) -> Result<()> {
    let staging = path.with_extension("json.tmp");
    let json = serde_json::to_vec(stored).context("encode HTTP generation")?;
    disk::save_durable(path, &staging, &json)
        .await
        .context("commit HTTP generation")
}
