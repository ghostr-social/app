//! Focus-window updates and playback URLs (plan §2 rows 3–4).

use crate::api::delivery_types::FfiFocusItem;
use crate::api::focus_mapping::{delivery_focus, focus_item};
use crate::api::runtime_registry::{self, EngineHandles};
use crate::engine::{DeliveryKind, VideoMeta};
use crate::video::delivery_events::{DeliveryFocus, FocusItem};
use anyhow::bail;
use flutter_rust_bridge::frb;

/// Replaces the ordered focus window (items include the current one),
/// refreshes the catalog, and wakes the delivery manager. `feed_id`
/// is carried for the phase-2 multi-feed surface; a single feed
/// exists today, so it is accepted but not yet interpreted.
#[frb]
pub async fn ffi_update_focus(
    feed_id: String,
    items: Vec<FfiFocusItem>,
    current_index: u32,
    watch_ms: u64,
) -> anyhow::Result<()> {
    let _ = feed_id;
    let focus = delivery_focus(&items, current_index, watch_ms)?;
    let engine = runtime_registry::engine()?;
    engine.tracked.replace(progressive_entries(&focus));
    engine.gateway.delivery().update_focus(focus);
    Ok(())
}

/// Loopback playback URL for one item, registering it with the
/// gateway when unknown. Downloads for a post are still driven by the
/// focus window, which includes the current item by contract. HLS
/// items are refused: HLS playback stays on the session-owning
/// `ffi_acquire_hls_playback`, so one URL scheme exists per delivery
/// kind instead of a half-working progressive URL for HLS.
#[frb]
pub async fn ffi_playback_url(item: FfiFocusItem) -> anyhow::Result<String> {
    let mapped = focus_item(&item)?;
    if mapped.meta.delivery == DeliveryKind::Hls {
        bail!("HLS items have no progressive URL; use ffi_acquire_hls_playback");
    }
    let engine = runtime_registry::engine()?;
    register_progressive(&engine, &mapped);
    Ok(progressive_url(&engine.endpoint, mapped.post.as_str()))
}

/// Only progressive posts are catalogued, downloaded, and watched;
/// HLS posts ride in the window for scroll distances alone.
fn progressive_entries(focus: &DeliveryFocus) -> Vec<(String, VideoMeta)> {
    focus
        .items
        .iter()
        .filter(|item| item.meta.delivery == DeliveryKind::Progressive)
        .map(|item| (item.post.as_str().to_owned(), item.meta.clone()))
        .collect()
}

fn register_progressive(engine: &EngineHandles, item: &FocusItem) {
    engine
        .gateway
        .progressive()
        .posts
        .insert(item.post.as_str());
    engine
        .tracked
        .insert(item.post.as_str().to_owned(), item.meta.clone());
}

pub(crate) fn progressive_url(endpoint: &str, id: &str) -> String {
    format!("http://{endpoint}/video.mp4?id={id}")
}
