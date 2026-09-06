use crate::partial_range_disk as range_disk;
use anyhow::{Context as _, Result};
use ghostr_engine::representation::SourceGeneration;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
#[path = "partial_range_generation_disk/selection_test.rs"]
mod selection_test;

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
    let Ok(stored) = serde_json::from_str(&text) else {
        return Ok(None);
    };
    Ok(validated(stored))
}

pub async fn save(path: &Path, stored: &StoredGeneration) -> Result<()> {
    let staging = path.with_extension("json.tmp");
    let json = serde_json::to_vec(stored).context("encode sparse generation")?;
    range_disk::save_durable(path, &staging, &json)
        .await
        .context("commit sparse generation")
}

fn validated(mut stored: StoredGeneration) -> Option<StoredGeneration> {
    let generation = SourceGeneration::try_new(
        stored.generation.final_url(),
        stored.generation.strong_etag(),
        stored.generation.total_bytes(),
    )
    .ok()?
    .with_request_selection(stored.generation.request_selection());
    stored.generation = generation;
    Some(stored)
}
