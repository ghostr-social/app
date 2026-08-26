//! The write path. Bytes are planned, admitted, put on disk and only
//! then recorded, so a write the store cannot afford costs nothing: the
//! manifest on disk never describes bytes that did not land.

use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_store::cleanup_debt::CleanupScope;
use crate::partial_range_store::StoreAction;
use crate::partial_range_store::{Entries, PartialRangeStore};
use anyhow::{bail, Context as _, Result};
use core::ops::Range;

/// The manifest a write would produce and the bytes it would add.
struct PlannedWrite {
    manifest: RangeManifest,
    added: u64,
    checksum_span: Range<u64>,
}

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when range geometry, capacity admission, or persistence fails.
    pub async fn write_range(&self, key: &str, offset: u64, bytes: &[u8]) -> Result<()> {
        let _update = self.update_key(key).await?;
        let mut entries = self.entries.lock().await;
        self.write_range_locked(&mut entries, key, offset, bytes)
            .await
    }

    pub(super) async fn write_range_locked(
        &self,
        entries: &mut Entries,
        key: &str,
        offset: u64,
        bytes: &[u8],
    ) -> Result<()> {
        self.write_range_locked_with_action(entries, key, None, offset, bytes)
            .await
    }

    pub(super) async fn write_range_locked_for_action(
        &self,
        entries: &mut Entries,
        key: &str,
        action: &StoreAction,
        offset: u64,
        bytes: &[u8],
    ) -> Result<()> {
        self.write_range_locked_with_action(entries, key, Some(action), offset, bytes)
            .await
    }

    async fn write_range_locked_with_action(
        &self,
        entries: &mut Entries,
        key: &str,
        action: Option<&StoreAction>,
        offset: u64,
        bytes: &[u8],
    ) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let end = offset
            .checked_add(bytes.len() as u64)
            .context("partial range end overflows")?;
        let plan = self.plan_write(entries, key, offset..end).await?;
        let debt = plan.added;
        let reserved = self.consume_if_action(action, plan.added).await?;
        if let Err(error) = self.make_room(entries, key, plan.added).await {
            self.restore_if_action(action, reserved).await;
            return Err(error);
        }
        let result = self.persist_write(entries, key, offset, bytes, plan).await;
        if result.is_err() && self.discard(entries, key).await.is_err() {
            self.record_cleanup_debt(key, CleanupScope::CanonicalDirty, action.cloned(), debt)
                .await?;
        }
        result
    }

    async fn persist_write(
        &self,
        entries: &mut Entries,
        key: &str,
        offset: u64,
        bytes: &[u8],
        mut plan: PlannedWrite,
    ) -> Result<()> {
        let path = self.paths.partial(key);
        disk::write_at(&path, offset, bytes).await?;
        let spans = core::slice::from_ref(&plan.checksum_span);
        for (span, checksum) in disk::checksum_blocks(&path, spans).await? {
            plan.manifest.record_checksum(span, checksum)?;
        }
        disk::save_manifest(&self.paths.manifest(key), &plan.manifest).await?;
        self.record_write(entries, key, plan).await
    }

    async fn consume_if_action(&self, action: Option<&StoreAction>, bytes: u64) -> Result<u64> {
        match action {
            Some(action) => self.consume_action(action, bytes).await,
            None => Ok(0),
        }
    }

    async fn restore_if_action(&self, action: Option<&StoreAction>, bytes: u64) {
        if let Some(action) = action {
            self.restore_action(action, bytes).await;
        }
    }

    /// # Errors
    ///
    /// Returns an error when the object is finalized or its length cannot be persisted safely.
    pub async fn set_total_len(&self, key: &str, len: u64) -> Result<()> {
        let _update = self.update_key(key).await?;
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
        let checksum_span = manifest.checksum_span_for_write(span.clone());
        manifest.insert(span)?;
        let added = manifest.covered_bytes() - entry.manifest.covered_bytes();
        Ok(PlannedWrite {
            manifest,
            added,
            checksum_span,
        })
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
