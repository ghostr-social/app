use super::PartialRangeStore;
use ghostr_engine::representation::RepresentationBinding;
use std::ops::Range;

#[cfg(test)]
mod tests;

pub enum RepresentationRead {
    Present(Vec<u8>),
    Missing,
    Superseded,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContentRevision(pub(super) u64);

impl PartialRangeStore {
    pub async fn read_for_representation(
        &self,
        binding: &RepresentationBinding,
        span: Range<u64>,
    ) -> anyhow::Result<RepresentationRead> {
        let Some(revision) = self.revision_for_binding(binding).await? else {
            return Ok(RepresentationRead::Superseded);
        };
        let read = self.read_range(binding.post().as_str(), span).await;
        self.finish_representation_read(binding, revision, read)
            .await
    }

    async fn finish_representation_read(
        &self,
        binding: &RepresentationBinding,
        revision: ContentRevision,
        read: anyhow::Result<Option<Vec<u8>>>,
    ) -> anyhow::Result<RepresentationRead> {
        if self.revision_for_binding(binding).await? != Some(revision) {
            return Ok(RepresentationRead::Superseded);
        }
        let read = read?;
        Ok(read.map_or(RepresentationRead::Missing, RepresentationRead::Present))
    }

    pub async fn stream_snapshot(
        &self,
        key: &str,
    ) -> (Option<RepresentationBinding>, ContentRevision) {
        let Ok(_update) = self.observe_key(key).await else {
            return (None, self.current_content_revision(key).await);
        };
        let binding = self.representations.lock().await.get(key).cloned();
        let revision = self.current_content_revision(key).await;
        (binding, revision)
    }

    pub async fn read_for_stream(
        &self,
        key: &str,
        binding: Option<&RepresentationBinding>,
        revision: ContentRevision,
        span: Range<u64>,
    ) -> anyhow::Result<RepresentationRead> {
        if !self.stream_is_current(key, binding, revision).await? {
            return Ok(RepresentationRead::Superseded);
        }
        let read = self.read_range(key, span).await;
        self.finish_stream_read(key, binding, revision, read).await
    }

    async fn finish_stream_read(
        &self,
        key: &str,
        binding: Option<&RepresentationBinding>,
        revision: ContentRevision,
        read: anyhow::Result<Option<Vec<u8>>>,
    ) -> anyhow::Result<RepresentationRead> {
        if !self.stream_is_current(key, binding, revision).await? {
            return Ok(RepresentationRead::Superseded);
        }
        let read = read?;
        Ok(read.map_or(RepresentationRead::Missing, RepresentationRead::Present))
    }

    async fn revision_for_binding(
        &self,
        binding: &RepresentationBinding,
    ) -> anyhow::Result<Option<ContentRevision>> {
        let _update = self.observe_key(binding.post().as_str()).await?;
        if !self.representation_is_current(binding).await {
            return Ok(None);
        }
        Ok(Some(
            self.current_content_revision(binding.post().as_str()).await,
        ))
    }

    /// Instantaneous authority check. A caller that waits after `true` must
    /// arm [`PartialRangeStore::change_notifier`] before checking.
    pub async fn stream_is_current(
        &self,
        key: &str,
        binding: Option<&RepresentationBinding>,
        revision: ContentRevision,
    ) -> anyhow::Result<bool> {
        let _update = self.observe_key(key).await?;
        let current = self.representations.lock().await.get(key).cloned();
        Ok(current.as_ref() == binding && self.current_content_revision(key).await == revision)
    }
}
