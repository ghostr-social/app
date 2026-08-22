//! Pure mapping from FFI payloads to engine types. No IO, no state —
//! fully covered by the unit tests in `crate::api::tests`.

use crate::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use crate::api::focus_control::{FfiFocusTransition, FfiTransportRescue, FfiTransportRescueReason};
use crate::engine::{DeliveryKind, PostId, PreviewDescriptor, VideoMeta};
use anyhow::{bail, Result};
use ghostr_delivery::delivery_events::{
    DeliveryFocus, FocusGeneration, FocusItem, FocusPreview, FocusTransition, TransportRescue,
    TransportRescueReason,
};

impl From<FfiFocusTransition> for FocusTransition {
    fn from(value: FfiFocusTransition) -> Self {
        match value {
            FfiFocusTransition::UserNavigation => Self::UserNavigation,
            FfiFocusTransition::RosterChange => Self::RosterChange,
            FfiFocusTransition::TransportRescue => Self::TransportRescue,
        }
    }
}

impl From<FfiMediaDelivery> for DeliveryKind {
    fn from(delivery: FfiMediaDelivery) -> Self {
        match delivery {
            FfiMediaDelivery::Progressive => Self::Progressive,
            FfiMediaDelivery::Hls => Self::Hls,
        }
    }
}

impl From<FfiTransportRescueReason> for TransportRescueReason {
    fn from(reason: FfiTransportRescueReason) -> Self {
        match reason {
            FfiTransportRescueReason::EtaUnavailable => Self::EtaUnavailable,
            FfiTransportRescueReason::EtaTooLong => Self::EtaTooLong,
            FfiTransportRescueReason::DeliveryFailed => Self::DeliveryFailed,
            FfiTransportRescueReason::GraceExpired => Self::GraceExpired,
        }
    }
}

/// Post ids double as partial-store keys and gateway `?id=` values.
pub(crate) fn validate_post_id(id: &str) -> Result<()> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_');
    if !id.is_empty() && id.chars().all(allowed) {
        return Ok(());
    }
    bail!("post ids must be non-empty and limited to [A-Za-z0-9_-]")
}

pub(crate) fn focus_item(item: &FfiFocusItem) -> Result<FocusItem> {
    validate_post_id(&item.post_id)?;
    Ok(FocusItem {
        post: PostId::new(item.post_id.clone()),
        meta: video_meta(item)?,
    })
}

fn video_meta(item: &FfiFocusItem) -> Result<VideoMeta> {
    Ok(VideoMeta {
        urls: item.urls.clone(),
        delivery: item.delivery.into(),
        sha256: item.sha256.clone(),
        size_bytes: item.size_bytes,
        duration_ms: item.duration_ms,
    })
}

/// Maps the whole window atomically: one bad item rejects the call so
/// the engine never sees a partially valid focus.
pub(crate) fn delivery_focus(
    items: &[FfiFocusItem],
    current_index: u32,
    watch_ms: u64,
    generation: u64,
    transition: FfiFocusTransition,
    rescue: Option<FfiTransportRescue>,
) -> Result<DeliveryFocus> {
    let Some(generation) = FocusGeneration::try_new(generation) else {
        bail!("focus generation must be positive");
    };
    let previews = items.iter().filter_map(focus_preview).collect::<Vec<_>>();
    let items = items.iter().map(focus_item).collect::<Result<Vec<_>>>()?;
    let transition: FocusTransition = transition.into();
    if (transition == FocusTransition::TransportRescue) != rescue.is_some() {
        bail!("transport rescue transition requires rescue context");
    }
    Ok(DeliveryFocus {
        items,
        previews,
        current_index: current_index as usize,
        watch_ms,
        generation,
        transition,
        rescue: rescue.map(|rescue| TransportRescue {
            reason: rescue.reason.into(),
            rank_displacement: rescue.rank_displacement,
            wait_ms: rescue.wait_ms,
        }),
    })
}

fn focus_preview(item: &FfiFocusItem) -> Option<FocusPreview> {
    Some(FocusPreview {
        post: PostId::new(item.post_id.clone()),
        descriptor: PreviewDescriptor::inline_blurhash(item.blurhash.as_deref()?)?,
    })
}
