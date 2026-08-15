//! Pure mapping from FFI payloads to engine types. No IO, no state —
//! fully covered by the unit tests in `crate::api::tests`.

use crate::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use crate::api::focus_control::FfiFocusTransition;
use crate::engine::{DeliveryKind, PostId, VideoMeta};
use anyhow::{bail, Result};
use ghostr_delivery::delivery_events::{
    DeliveryFocus, FocusGeneration, FocusItem, FocusTransition,
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
) -> Result<DeliveryFocus> {
    let Some(generation) = FocusGeneration::try_new(generation) else {
        bail!("focus generation must be positive");
    };
    let items = items.iter().map(focus_item).collect::<Result<Vec<_>>>()?;
    Ok(DeliveryFocus {
        items,
        current_index: current_index as usize,
        watch_ms,
        generation,
        transition: transition.into(),
    })
}
