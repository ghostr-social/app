use super::{ContentRevision, PartialRangeStore};
use crate::partial_range_completion::Completion;
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::representation::RepresentationBinding;

mod record;
mod recovery;
mod transaction;

pub struct TransformFence {
    binding: RepresentationBinding,
    revision: ContentRevision,
}

impl TransformFence {
    pub const fn new(binding: RepresentationBinding, revision: ContentRevision) -> Self {
        Self { binding, revision }
    }
}

pub struct TransformPublication {
    fence: TransformFence,
    kind: TransformKind,
    output: Vec<u8>,
}

impl TransformPublication {
    pub fn try_new(
        fence: TransformFence,
        kind: TransformKind,
        output: Vec<u8>,
        maximum_bytes: u64,
    ) -> Result<Self> {
        ensure!(!output.is_empty(), "transform output must not be empty");
        ensure!(
            output.len() as u64 <= maximum_bytes,
            "transform output exceeds its selected envelope"
        );
        Ok(Self {
            fence,
            kind,
            output,
        })
    }

    const fn output_bytes(&self) -> u64 {
        self.output.len() as u64
    }
}

impl PartialRangeStore {
    pub async fn publish_transform(&self, publication: TransformPublication) -> Result<bool> {
        let key = publication.fence.binding.post().as_str().to_owned();
        let _lease = self.lease(&key);
        let _update = self.update_key(&key).await?;
        let Some(old_bytes) = self.transform_input_bytes(&publication).await? else {
            return Ok(false);
        };
        let staged = publication.output_bytes();
        self.require_headroom(staged).await?;
        let prepared = transaction::stage(&self.paths, &key, publication).await?;
        self.credit(staged).await;
        self.capacity.spent(staged).await;
        if let Err(error) = transaction::commit(&self.paths, &key).await {
            transaction::rollback(&self.paths, &key).await?;
            self.release(staged).await;
            return Err(error);
        }
        self.install_transform(&key, old_bytes, prepared).await;
        Ok(true)
    }

    async fn transform_input_bytes(
        &self,
        publication: &TransformPublication,
    ) -> Result<Option<u64>> {
        let key = publication.fence.binding.post().as_str();
        if !self
            .representation_is_current(&publication.fence.binding)
            .await
            || self.current_content_revision(key).await != publication.fence.revision
        {
            return Ok(None);
        }
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok((entry.completion.is_some() && entry.manifest.is_complete()).then_some(entry.accounted))
    }

    async fn install_transform(&self, key: &str, old_bytes: u64, prepared: transaction::Prepared) {
        let output_bytes = prepared.output_bytes();
        let mut entries = self.entries.lock().await;
        let entry = entries.get_mut(key).expect("transform input entry");
        entry.manifest = prepared.manifest;
        entry.accounted = output_bytes;
        entry.completion = Some(Completion::Unverified);
        entry.touched = self.tick();
        self.representations
            .lock()
            .await
            .insert(key.to_owned(), prepared.binding);
        self.source_generations.lock().await.remove(key);
        self.selected().remove(key);
        self.advance_content_revision(key).await;
        self.release(old_bytes).await;
        self.changed.notify_waiters();
    }

    pub(super) async fn recover_transform_locked(&self, key: &str) -> Result<()> {
        recovery::recover(&self.paths, key).await
    }

    pub(super) async fn restored_transform_binding(
        &self,
        input: &RepresentationBinding,
        stored: Option<&str>,
    ) -> Result<Option<RepresentationBinding>> {
        record::restore_binding(&self.paths, input, stored).await
    }
}
