use crate::partial_range_disk as disk;
use anyhow::{bail, Context, Result};
use std::path::Path;

pub async fn load(path: &Path) -> Result<Option<String>> {
    match tokio::fs::read_to_string(path).await {
        Ok(value) => validate(value.trim()).map(|value| Some(value.to_owned())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("read stored representation identity"),
    }
}

pub async fn save(path: &Path, identity: &str) -> Result<()> {
    validate(identity)?;
    let staging = path.with_extension("representation.tmp");
    disk::save_durable(path, &staging, identity.as_bytes())
        .await
        .context("commit stored representation identity")
}

fn validate(value: &str) -> Result<&str> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(value);
    }
    bail!("stored representation identity is invalid")
}
