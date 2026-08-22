//! R→D delivery events (plan §2 row 5). A watcher task derives
//! per-post readiness and progress from the partial store; the
//! manager's writes wake it through the store's change notifier, so
//! every manager state change surfaces without polling.

use crate::api::delivery::snapshots::{
    compute_snapshot, error_event, event_for, hls_snapshot, DeliverySnapshot, SnapshotInput,
};
use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::runtime::registry;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::engine::budget::params_for;
use crate::engine::{ByteRange, DeliveryKind, EngineParams, PostId, VideoMeta};
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::collections::HashMap;
use std::sync::Arc;

/// Where watcher events go; lets tests observe without a Dart sink.
pub(crate) trait EventOut: Send + 'static {
    /// Returns `false` once the receiver is gone: the watcher stops.
    fn send(&self, event: FfiDeliveryEvent) -> bool;
}

impl EventOut for StreamSink<FfiDeliveryEvent> {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        self.add(event).is_ok()
    }
}

/// Subscribes to per-post delivery events. Each subscription runs its
/// own watcher and first reports a readiness baseline for every
/// watched post; the stream ends when the Dart side cancels it.
#[frb]
pub async fn ffi_delivery_events(sink: StreamSink<FfiDeliveryEvent>) -> anyhow::Result<()> {
    let engine = registry::engine()?;
    let store = engine.gateway.progressive().store.clone();
    let segmented = engine.gateway.segmented();
    tokio::spawn(watch_delivery(
        sink,
        store,
        segmented,
        engine.tracked.clone(),
    ));
    Ok(())
}

pub(crate) async fn watch_delivery(
    out: impl EventOut,
    store: Arc<PartialRangeStore>,
    segmented: SegmentedCache,
    tracked: TrackedItems,
) {
    let store_changed = store.change_notifier();
    let items_changed = tracked.notifier();
    let segmented_changed = segmented.notifier();
    let mut emitted: HashMap<String, DeliverySnapshot> = HashMap::new();
    loop {
        let store_wake = store_changed.notified();
        let items_wake = items_changed.notified();
        let segmented_wake = segmented_changed.notified();
        tokio::pin!(store_wake, items_wake, segmented_wake);
        store_wake.as_mut().enable();
        items_wake.as_mut().enable();
        segmented_wake.as_mut().enable();
        if !emit_pass(&out, &store, &segmented, &tracked, &mut emitted).await {
            return;
        }
        tokio::select! {
            _ = store_wake => {},
            _ = items_wake => {},
            _ = segmented_wake => {},
        }
    }
}

async fn emit_pass(
    out: &impl EventOut,
    store: &PartialRangeStore,
    segmented: &SegmentedCache,
    tracked: &TrackedItems,
    emitted: &mut HashMap<String, DeliverySnapshot>,
) -> bool {
    let params = params_for(tracked.level(), EngineParams::default());
    let entries = tracked.snapshot();
    emitted.retain(|id, _| entries.iter().any(|(known, _)| known == id));
    for (id, meta) in &entries {
        let mut pass = Pass {
            store,
            segmented,
            params: &params,
            emitted,
        };
        if !emit_post(out, &mut pass, id, meta).await {
            return false;
        }
    }
    true
}

struct Pass<'a> {
    store: &'a PartialRangeStore,
    segmented: &'a SegmentedCache,
    params: &'a EngineParams,
    emitted: &'a mut HashMap<String, DeliverySnapshot>,
}

async fn emit_post(out: &impl EventOut, pass: &mut Pass<'_>, id: &str, meta: &VideoMeta) -> bool {
    if meta.delivery == DeliveryKind::Hls {
        return emit_hls(out, pass, id);
    }
    let (ranges, stored_total) = match store_view(pass.store, id, meta).await {
        Ok(view) => view,
        Err(error) => return out.send(error_event(id, error.to_string())),
    };
    let post = PostId::new(id);
    let input = SnapshotInput {
        meta,
        ranges: &ranges,
        stored_total,
        params: pass.params,
    };
    let current = compute_snapshot(&post, input);
    let event = event_for(id, pass.emitted.get(id), current.clone());
    pass.emitted.insert(id.to_owned(), current);
    match event {
        Some(event) => out.send(event),
        None => true,
    }
}

fn emit_hls(out: &impl EventOut, pass: &mut Pass<'_>, id: &str) -> bool {
    let current = hls_snapshot(pass.segmented.snapshot(id));
    let event = event_for(id, pass.emitted.get(id), current.clone());
    pass.emitted.insert(id.to_owned(), current);
    event.is_none_or(|event| out.send(event))
}

async fn store_view(
    store: &PartialRangeStore,
    id: &str,
    meta: &VideoMeta,
) -> anyhow::Result<(Vec<ByteRange>, Option<u64>)> {
    let snapshot = store.media_snapshot(id).await?;
    if !snapshot
        .binding()
        .is_some_and(|binding| binding.matches_meta(meta))
    {
        return Ok((Vec::new(), None));
    }
    let ranges = snapshot
        .ranges()
        .iter()
        .map(|span| ByteRange::new(span.start, span.end))
        .collect();
    Ok((ranges, snapshot.total_len()))
}
