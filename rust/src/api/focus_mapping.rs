//! Pure mapping from FFI payloads to engine types. No IO, no state —
//! fully covered by the unit tests in `crate::api::tests`.

use crate::api::delivery_types::FfiFocusItem;
use crate::engine::{DataUsageLevel, DeliveryKind, PostId, VideoMeta};
use crate::video::delivery_events::{DeliveryFocus, FocusItem};
use anyhow::{bail, Result};

pub(crate) fn parse_data_usage(raw: &str) -> Result<DataUsageLevel> {
    match raw {
        "conservative" => Ok(DataUsageLevel::Conservative),
        "balanced" => Ok(DataUsageLevel::Balanced),
        "aggressive" => Ok(DataUsageLevel::Aggressive),
        other => bail!("unknown data usage level: {other}"),
    }
}

pub(crate) fn parse_delivery_kind(raw: &str) -> Result<DeliveryKind> {
    match raw {
        "progressive" => Ok(DeliveryKind::Progressive),
        "hls" => Ok(DeliveryKind::Hls),
        other => bail!("unknown delivery kind: {other}"),
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
        delivery: parse_delivery_kind(&item.delivery)?,
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
) -> Result<DeliveryFocus> {
    let items = items.iter().map(focus_item).collect::<Result<Vec<_>>>()?;
    Ok(DeliveryFocus {
        items,
        current_index: current_index as usize,
        watch_ms,
    })
}
