//! Focus-window updates and playback URLs (plan §2 rows 3–4).

use crate::api::delivery::focus_mapping::{delivery_focus, focus_item};
use crate::api::delivery_types::FfiFocusItem;
use crate::api::runtime::registry::{self, EngineHandles};
use crate::engine::{DeliveryKind, VideoMeta};
use anyhow::bail;
use flutter_rust_bridge::frb;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission, FocusItem};

/// One atomic, monotonically versioned focus write from Flutter.
#[derive(Debug)]
pub struct FfiFocusUpdate {
    pub feed_id: String,
    pub items: Vec<FfiFocusItem>,
    pub current_index: u32,
    pub watch_ms: u64,
    pub generation: u64,
}

/// Replaces the ordered focus window (items include the current one),
/// refreshes the catalog, and wakes the delivery manager. `feed_id`
/// is carried for a future multi-feed surface; one feed exists today.
#[frb]
pub async fn ffi_update_focus(update: FfiFocusUpdate) -> anyhow::Result<()> {
    let _ = update.feed_id;
    let focus = delivery_focus(
        &update.items,
        update.current_index,
        update.watch_ms,
        update.generation,
    )?;
    let engine = registry::engine()?;
    accept_focus(&engine, focus)
}

fn accept_focus(engine: &EngineHandles, focus: DeliveryFocus) -> anyhow::Result<()> {
    let generation = focus.generation;
    let entries = progressive_entries(&focus);
    match engine.gateway.delivery().update_focus(focus) {
        FocusAdmission::Accepted if engine.tracked.replace_focus(generation, entries) => Ok(()),
        FocusAdmission::Accepted | FocusAdmission::Stale => {
            bail!("focus generation was superseded")
        }
        FocusAdmission::Closed => bail!("delivery manager is unavailable"),
    }
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
    let engine = registry::engine()?;
    register_progressive(&engine, &mapped);
    let capability = engine.gateway.issue_progressive(mapped.post.as_str()).await;
    Ok(progressive_url(
        &engine.endpoint,
        mapped.post.as_str(),
        capability.as_str(),
    ))
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
        .cache
        .insert(item.post.as_str());
    engine
        .tracked
        .insert(item.post.as_str().to_owned(), item.meta.clone());
}

pub(crate) fn progressive_url(endpoint: &str, id: &str, capability: &str) -> String {
    format!("http://{endpoint}/video.mp4?id={id}&cap={capability}")
}
