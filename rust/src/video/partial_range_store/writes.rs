//! The write path. Bytes are planned, admitted, put on disk and only
//! then recorded, so a write the store cannot afford costs nothing: the
//! manifest on disk never describes bytes that did not land.

use crate::video::partial_range_disk as disk;
use crate::video::partial_range_manifest::RangeManifest;
use crate::video::partial_range_store::{Entries, PartialRangeStore};
use anyhow::{bail, Context, Result};
use std::ops::Range;

/// The manifest a write would produce and the bytes it would add.
pub(crate) struct PlannedWrite {
    manifest: RangeManifest,
    added: u64,
}

impl PartialRangeStore {
    pub async fn write_range(&self, key: &str, offset: u64, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let end = offset
            .checked_add(bytes.len() as u64)
            .context("partial range end overflows")?;
        let mut entries = self.entries.lock().await;
        let plan = self.plan_write(&mut entries, key, offset..end).await?;
        self.make_room(&mut entries, key, plan.added).await?;
        disk::write_at(&self.paths.partial(key), offset, bytes).await?;
        disk::save_manifest(&self.paths.manifest(key), &plan.manifest).await?;
        self.record_write(&mut entries, key, plan).await
    }

    pub async fn set_total_len(&self, key: &str, len: u64) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completion.is_some() {
            bail!("cannot resize a finalized video");
        }
        entry.manifest.set_total_len(len)?;
        disk::save_manifest(&self.paths.manifest(key), &entry.manifest).await?;
        self.changed.notify_waiters();
        Ok(())
    }

    /// What the write would leave behind, refusing a finalized key. The
    /// manifest is only a plan here: nothing touches disk until the
    /// bytes are admitted.
    async fn plan_write(
        &self,
        entries: &mut Entries,
        key: &str,
        span: Range<u64>,
    ) -> Result<PlannedWrite> {
        let entry = self.entry(entries, key).await?;
        if entry.completion.is_some() {
            bail!("cannot write into a finalized video");
        }
        let mut manifest = entry.manifest.clone();
        manifest.insert(span)?;
        let added = manifest.covered_bytes() - entry.manifest.covered_bytes();
        Ok(PlannedWrite { manifest, added })
    }

    /// Bookkeeping for bytes that are already on disk. The capacity
    /// model is told what was spent, so the reserve stays exact between
    /// free-space measurements.
    async fn record_write(
        &self,
        entries: &mut Entries,
        key: &str,
        plan: PlannedWrite,
    ) -> Result<()> {
        let touched = self.tick();
        let entry = self.entry(entries, key).await?;
        entry.manifest = plan.manifest;
        entry.accounted += plan.added;
        entry.touched = touched;
        self.credit(plan.added).await;
        self.capacity.spent(plan.added).await;
        self.changed.notify_waiters();
        Ok(())
    }
}
