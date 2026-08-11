use super::PLAYBACK_SLICE_BYTES;
use crate::progressive::route::ProgressiveState;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::RepresentationRead;
use std::ops::Range;

pub(crate) struct StreamSource {
    key: String,
    binding: Option<RepresentationBinding>,
}

pub(super) enum ChunkRead {
    Present(Vec<u8>),
    Missing,
    Superseded,
}

impl StreamSource {
    pub(crate) fn new(key: String, binding: Option<RepresentationBinding>) -> Self {
        Self { key, binding }
    }

    pub(super) fn key(&self) -> &str {
        &self.key
    }
}

pub(super) async fn next_chunk(
    state: &ProgressiveState,
    source: &StreamSource,
    remaining: Range<u64>,
) -> anyhow::Result<ChunkRead> {
    let Some(span) = available_prefix(state, source.key(), remaining).await? else {
        return Ok(ChunkRead::Missing);
    };
    let Some(binding) = &source.binding else {
        return unbound_read(state, source.key(), span).await;
    };
    Ok(
        match state.store.read_for_representation(binding, span).await? {
            RepresentationRead::Present(bytes) => ChunkRead::Present(bytes),
            RepresentationRead::Missing => ChunkRead::Missing,
            RepresentationRead::Superseded => ChunkRead::Superseded,
        },
    )
}

async fn unbound_read(
    state: &ProgressiveState,
    key: &str,
    span: Range<u64>,
) -> anyhow::Result<ChunkRead> {
    Ok(match state.store.read_range(key, span).await? {
        Some(bytes) => ChunkRead::Present(bytes),
        None => ChunkRead::Missing,
    })
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
