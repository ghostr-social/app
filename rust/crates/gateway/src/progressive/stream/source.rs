use super::PLAYBACK_SLICE_BYTES;
use crate::progressive::route::ProgressiveState;
use core::ops::Range;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::{ContentRevision, RepresentationRead};

pub(crate) struct StreamSource {
    key: String,
    binding: Option<RepresentationBinding>,
    revision: ContentRevision,
}

pub(super) enum ChunkRead {
    Present(Vec<u8>),
    Missing,
    Superseded,
}

impl StreamSource {
    pub(crate) fn new(
        key: String,
        binding: Option<RepresentationBinding>,
        revision: ContentRevision,
    ) -> Self {
        Self {
            key,
            binding,
            revision,
        }
    }

    pub(super) fn key(&self) -> &str {
        &self.key
    }

    pub(super) fn binding(&self) -> Option<&RepresentationBinding> {
        self.binding.as_ref()
    }
}

pub(super) async fn next_chunk(
    state: &ProgressiveState,
    source: &StreamSource,
    remaining: Range<u64>,
) -> anyhow::Result<ChunkRead> {
    let Some(span) = available_prefix(state, source.key(), remaining).await? else {
        let current = state
            .store
            .stream_is_current(source.key(), source.binding.as_ref(), source.revision)
            .await?;
        return Ok(if current {
            ChunkRead::Missing
        } else {
            ChunkRead::Superseded
        });
    };
    Ok(
        match state
            .store
            .read_for_stream(source.key(), source.binding.as_ref(), source.revision, span)
            .await?
        {
            RepresentationRead::Present(bytes) => ChunkRead::Present(bytes),
            RepresentationRead::Missing => ChunkRead::Missing,
            RepresentationRead::Superseded => ChunkRead::Superseded,
        },
    )
}

async fn available_prefix(
    state: &ProgressiveState,
    key: &str,
    remaining: Range<u64>,
) -> anyhow::Result<Option<Range<u64>>> {
    let missing = state.store.missing_within(key, remaining.clone()).await?;
    let available_end = match missing.first() {
        Some(hole) if hole.start <= remaining.start => return Ok(None),
        Some(hole) => hole.start,
        None => remaining.end,
    };
    let end = available_end.min(remaining.start.saturating_add(PLAYBACK_SLICE_BYTES));
    Ok(Some(remaining.start..end))
}
