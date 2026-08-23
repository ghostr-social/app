use super::super::PreparationContext;
use super::CertifiedReadiness;
use crate::api::delivery_types::{FfiPlaybackPreparationAsset, FfiPlaybackPreparationReadiness};
use crate::api::focus_control::progressive_url;
use crate::engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;

pub(super) async fn project(
    context: &PreparationContext,
    post: &PostId,
    readiness: Option<CertifiedReadiness<'_>>,
) -> Option<FfiPlaybackPreparationAsset> {
    let meta = context.tracked.meta(post.as_str())?;
    let snapshot = validated_snapshot(context, post, &meta).await?;
    if readiness.is_some_and(|value| !value.certificate().still_valid_in(&snapshot)) {
        return None;
    }
    let capability = context.capabilities.issue(&snapshot).await.ok()?;
    let binding = snapshot.binding()?;
    Some(FfiPlaybackPreparationAsset {
        delivery_id: post.as_str().to_owned(),
        representation_id: binding.representation().fingerprint().to_owned(),
        source_representation_id: binding.source_representation().fingerprint().to_owned(),
        asset_id: capability.as_str().to_owned(),
        playback_url: progressive_url(&context.endpoint, post.as_str(), capability.as_str()),
        readiness: projected_readiness(readiness),
    })
}

async fn validated_snapshot(
    context: &PreparationContext,
    post: &PostId,
    meta: &VideoMeta,
) -> Option<StoredMediaSnapshot> {
    if meta.delivery != DeliveryKind::Progressive {
        return None;
    }
    let snapshot = context.store.media_snapshot(post.as_str()).await.ok()?;
    let binding = snapshot.binding()?;
    (snapshot.total_len().is_some()
        && binding.matches_source_meta(meta)
        && context.cache.matches_binding(post.as_str(), binding))
    .then_some(snapshot)
}

fn projected_readiness(
    readiness: Option<CertifiedReadiness<'_>>,
) -> FfiPlaybackPreparationReadiness {
    match readiness {
        Some(CertifiedReadiness::Ready(_)) => FfiPlaybackPreparationReadiness::Ready,
        Some(CertifiedReadiness::Structural(_)) => {
            FfiPlaybackPreparationReadiness::StructuralStartable
        }
        None => FfiPlaybackPreparationReadiness::Preparing,
    }
}
