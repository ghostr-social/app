use super::slice;
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::IntervalChecksum;
use anyhow::{Context, Result};
use std::ops::Range;
use std::sync::Arc;
use tokio::task::JoinSet;

const MAX_CHECKSUM_WORKERS: usize = 4;

pub(super) async fn matches(
    checksums: &[IntervalChecksum],
    bytes: Arc<[u8]>,
    envelope: Range<u64>,
) -> Result<bool> {
    let mut pending = checksums.iter().cloned();
    let mut jobs = JoinSet::new();
    for checksum in pending.by_ref().take(MAX_CHECKSUM_WORKERS) {
        launch(&mut jobs, checksum, Arc::clone(&bytes), envelope.clone());
    }
    while let Some(result) = jobs.join_next().await {
        if !result.context("join stored checksum verification")?? {
            jobs.abort_all();
            return Ok(false);
        }
        if let Some(checksum) = pending.next() {
            launch(&mut jobs, checksum, Arc::clone(&bytes), envelope.clone());
        }
    }
    Ok(true)
}

fn launch(
    jobs: &mut JoinSet<Result<bool>>,
    checksum: IntervalChecksum,
    bytes: Arc<[u8]>,
    envelope: Range<u64>,
) {
    jobs.spawn_blocking(move || {
        let interval = slice(&bytes, &envelope, &checksum.span())?;
        Ok(disk::sha256_bytes(interval) == checksum.digest())
    });
}
