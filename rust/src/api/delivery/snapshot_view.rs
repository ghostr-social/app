//! One generation-coherent progressive observation for delivery events.

use super::snapshots::DeliverySnapshotAuthority;
use crate::engine::{ByteRange, VideoMeta};
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::PartialRangeStore;

pub(crate) struct SnapshotViewContext<'a> {
    pub store: &'a PartialRangeStore,
    pub cache: &'a CacheRegistry,
    pub capabilities: &'a ProgressiveCapabilities,
}

pub(super) struct SnapshotView {
    pub ranges: Vec<ByteRange>,
    pub stored_total: Option<u64>,
    pub meta: Option<VideoMeta>,
    pub playback_blocked: bool,
    /// Every origin of the post is retired; the item cannot become startable.
    pub exhausted: bool,
    pub authority: Option<DeliverySnapshotAuthority>,
}

pub(super) async fn capture(
    context: SnapshotViewContext<'_>,
    id: &str,
    meta: &VideoMeta,
) -> anyhow::Result<SnapshotView> {
    let snapshot = context.store.media_snapshot(id).await?;
    let Some(binding) = snapshot.binding() else {
        return Ok(empty());
    };
    if !binding.matches_source_meta(meta) {
        return Ok(empty());
    }
    let Some(current) = context.cache.video_for_binding(id, binding) else {
        return Ok(empty());
    };
    let representation_id = binding.representation().fingerprint().to_owned();
    let playback_blocked = context.cache.is_playback_blocked(id, binding);
    let exhausted = !current.status.is_servable();
    let authority =
        context
            .capabilities
            .existing(&snapshot)
            .await
            .map(|asset| DeliverySnapshotAuthority {
                representation_id,
                asset_id: asset.as_str().to_owned(),
            });
    Ok(SnapshotView {
        ranges: snapshot
            .ranges()
            .iter()
            .map(|span| ByteRange::new(span.start, span.end))
            .collect(),
        stored_total: snapshot.total_len(),
        meta: Some(current.meta),
        playback_blocked,
        exhausted,
        authority,
    })
}

fn empty() -> SnapshotView {
    SnapshotView {
        ranges: Vec::new(),
        stored_total: None,
        meta: None,
        playback_blocked: false,
        exhausted: false,
        authority: None,
    }
}
