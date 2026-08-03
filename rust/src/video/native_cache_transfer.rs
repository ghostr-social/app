use crate::video::native_cache::NativeVideoCache;
use anyhow::{Error, Result};
use reqwest::Response;
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct TransferredVideo {
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug)]
struct TransferFailure {
    source: Error,
    reserved_bytes: u64,
}

impl Display for TransferFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.source, formatter)
    }
}

impl std::error::Error for TransferFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub fn reserved_bytes(error: &Error) -> u64 {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<TransferFailure>())
        .map_or(0, |failure| failure.reserved_bytes)
}

pub async fn transfer(
    response: &mut Response,
    file: &mut File,
    cache: &NativeVideoCache,
) -> Result<TransferredVideo> {
    let mut transferred = 0;
    let mut digest = Sha256::new();
    while let Some(chunk) = next_chunk(response, transferred).await? {
        let bytes = chunk.len() as u64;
        reserve_chunk(cache, bytes, transferred).await?;
        write_chunk(file, &chunk, transferred + bytes).await?;
        transferred += bytes;
        digest.update(&chunk);
    }
    Ok(TransferredVideo {
        bytes: transferred,
        sha256: format!("{:x}", digest.finalize()),
    })
}

async fn next_chunk(response: &mut Response, reserved: u64) -> Result<Option<bytes::Bytes>> {
    response
        .chunk()
        .await
        .map_err(|error| failure(error.into(), reserved))
}

async fn reserve_chunk(cache: &NativeVideoCache, bytes: u64, transferred: u64) -> Result<()> {
    cache
        .ensure_object_fits(transferred.saturating_add(bytes))
        .map_err(|error| failure(error, transferred))?;
    cache
        .reserve(bytes)
        .await
        .map_err(|error| failure(error, transferred))
}

async fn write_chunk(file: &mut File, chunk: &[u8], reserved: u64) -> Result<()> {
    file.write_all(chunk)
        .await
        .map_err(|error| failure(error.into(), reserved))
}

fn failure(source: Error, reserved_bytes: u64) -> Error {
    TransferFailure {
        source,
        reserved_bytes,
    }
    .into()
}
