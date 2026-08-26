use super::{ContentRevision, PartialRangeStore};
use crate::partial_range_completion::Completion;
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::representation::RepresentationBinding;

mod record;
mod recovery;
#[cfg(any(test, feature = "test"))]
mod test_support;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformPublicationOutcome {
    Published,
    Superseded,
    Cancelled,
}

impl TransformPublication {
    /// # Errors
    ///
    /// Returns an error when output is empty or exceeds its selected byte envelope.
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
    /// # Errors
    ///
    /// Returns an error when authorization, authority, or durable publication fails.
    pub async fn publish_transform_authorized<F>(
        &self,
        publication: TransformPublication,
        authorize: F,
    ) -> Result<TransformPublicationOutcome>
    where
        F: FnOnce() -> bool + Send,
    {
        let key = publication.fence.binding.post().as_str().to_owned();
        let _lease = self.lease(&key);
        let _update = self.update_key(&key).await?;
        self.publish_transform_locked(&key, publication, authorize)
            .await
    }

    async fn publish_transform_locked<F>(
        &self,
        key: &str,
        publication: TransformPublication,
        authorize: F,
    ) -> Result<TransformPublicationOutcome>
    where
        F: FnOnce() -> bool,
    {
        let Some(old_bytes) = self.transform_input_bytes(&publication).await? else {
            return Ok(TransformPublicationOutcome::Superseded);
        };
        let staged = publication.output_bytes();
        self.require_headroom(staged).await?;
        let prepared = transaction::stage(&self.paths, key, publication).await?;
        if !authorize() {
            transaction::discard_staging(&self.paths, key).await?;
            return Ok(TransformPublicationOutcome::Cancelled);
        }
        self.commit_transform(key, old_bytes, prepared).await?;
        Ok(TransformPublicationOutcome::Published)
    }

    async fn commit_transform(
        &self,
        key: &str,
        old_bytes: u64,
        prepared: transaction::Prepared,
    ) -> Result<()> {
        let staged = prepared.output_bytes();
        self.credit(staged).await;
        self.capacity.spent(staged).await;
        if let Err(error) = transaction::commit(&self.paths, key).await {
            transaction::rollback(&self.paths, key).await?;
            self.release(staged).await;
            return Err(error);
        }
        self.install_transform(key, old_bytes, prepared).await;
        Ok(())
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
