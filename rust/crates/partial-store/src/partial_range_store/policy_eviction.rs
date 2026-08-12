//! Exact policy-selected sparse eviction, serialized with writes.

use super::{Entries, PartialRangeStore};
use crate::partial_range_disk::{self as disk, Entry};
use crate::partial_range_manifest::RangeManifest;
use anyhow::{ensure, Context, Result};
use std::ops::Range;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const COPY_BUFFER_BYTES: usize = 64 * 1024;

struct EvictionPlan {
    manifest: RangeManifest,
    freed: u64,
    accounted: u64,
    completed: bool,
}

impl PartialRangeStore {
    pub async fn evict_ranges(&self, key: &str, ranges: &[Range<u64>]) -> Result<u64> {
        if ranges.is_empty() || self.leases.held(key) {
            return Ok(0);
        }
        let mut entries = self.entries.lock().await;
        let plan = plan(self.entry(&mut entries, key).await?, ranges);
        if plan.freed == 0 {
            return Ok(0);
        }
        if plan.completed {
            return self.evict_complete(&mut entries, key, plan).await;
        }
        self.rewrite_partial(key, &plan.manifest).await?;
        self.record_eviction(&mut entries, key, plan).await
    }

    async fn evict_complete(
        &self,
        entries: &mut Entries,
        key: &str,
        plan: EvictionPlan,
    ) -> Result<u64> {
        ensure!(
            plan.freed == plan.accounted,
            "cannot split a finalized video"
        );
        self.discard(entries, key).await?;
        self.changed.notify_waiters();
        Ok(plan.freed)
    }

    async fn rewrite_partial(&self, key: &str, manifest: &RangeManifest) -> Result<()> {
        let source = self.paths.partial(key);
        let staging = self.paths.partial_staging(key);
        let ranges = manifest.ranges();
        rewrite_sparse(&source, &staging, &ranges).await?;
        disk::save_manifest(&self.paths.manifest(key), manifest).await?;
        match ranges.is_empty() {
            true => disk::remove_if_present(&source).await?,
            false => tokio::fs::rename(staging, source)
                .await
                .context("commit policy-evicted partial video")?,
        }
        Ok(())
    }

    async fn record_eviction(
        &self,
        entries: &mut Entries,
        key: &str,
        plan: EvictionPlan,
    ) -> Result<u64> {
        let entry = entries.get_mut(key).context("evicted entry present")?;
        entry.manifest = plan.manifest;
        entry.accounted = entry.accounted.saturating_sub(plan.freed);
        entry.touched = self.tick();
        self.release(plan.freed).await;
        self.changed.notify_waiters();
        Ok(plan.freed)
    }
}

fn plan(entry: &Entry, ranges: &[Range<u64>]) -> EvictionPlan {
    let mut manifest = entry.manifest.clone();
    let freed = ranges.iter().map(|range| manifest.remove(range)).sum();
    EvictionPlan {
        manifest,
        freed,
        accounted: entry.accounted,
        completed: entry.completion.is_some(),
    }
}

async fn rewrite_sparse(source: &Path, staging: &Path, ranges: &[Range<u64>]) -> Result<()> {
    disk::remove_if_present(staging).await?;
    if ranges.is_empty() {
        return Ok(());
    }
    let mut input = tokio::fs::File::open(source)
        .await
        .context("open policy-evicted partial video")?;
    let mut output = tokio::fs::File::create(staging)
        .await
        .context("create policy-evicted staging video")?;
    for range in ranges {
        copy_range(&mut input, &mut output, range).await?;
    }
    output.flush().await.context("flush policy-evicted video")
}

async fn copy_range(
    input: &mut tokio::fs::File,
    output: &mut tokio::fs::File,
    range: &Range<u64>,
) -> Result<()> {
    input.seek(std::io::SeekFrom::Start(range.start)).await?;
    output.seek(std::io::SeekFrom::Start(range.start)).await?;
    let mut left = range.end.saturating_sub(range.start);
    let mut buffer = vec![0; COPY_BUFFER_BYTES];
    while left > 0 {
        let length = left.min(buffer.len() as u64) as usize;
        input.read_exact(&mut buffer[..length]).await?;
        output.write_all(&buffer[..length]).await?;
        left -= length as u64;
    }
    Ok(())
}
