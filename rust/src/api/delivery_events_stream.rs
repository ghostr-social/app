//! R→D delivery events (plan §2 row 5). A watcher task derives
//! per-post readiness and progress from the partial store; the
//! manager's writes wake it through the store's change notifier, so
//! every manager state change surfaces without polling.

use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::event_snapshots::{
    compute_snapshot, error_event, event_for, DeliverySnapshot, SnapshotInput,
};
use crate::api::runtime_registry;
use crate::api::tracked_items::TrackedItems;
use crate::engine::budget::params_for;
use crate::engine::{ByteRange, EngineParams, PostId, VideoMeta};
use crate::frb_generated::StreamSink;
use crate::video::partial_range_store::PartialRangeStore;
use flutter_rust_bridge::frb;
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
    let engine = runtime_registry::engine()?;
    let store = engine.gateway.progressive().store.clone();
    tokio::spawn(watch_delivery(sink, store, engine.tracked.clone()));
    Ok(())
}

pub(crate) async fn watch_delivery(
    out: impl EventOut,
    store: Arc<PartialRangeStore>,
    tracked: TrackedItems,
) {
    let store_changed = store.change_notifier();
    let items_changed = tracked.notifier();
    let mut emitted: HashMap<String, DeliverySnapshot> = HashMap::new();
    loop {
        let store_wake = store_changed.notified();
        let items_wake = items_changed.notified();
        tokio::pin!(store_wake, items_wake);
        store_wake.as_mut().enable();
        items_wake.as_mut().enable();
        if !emit_pass(&out, &store, &tracked, &mut emitted).await {
            return;
        }
        tokio::select! { _ = store_wake => {}, _ = items_wake => {} }
    }
}

async fn emit_pass(
    out: &impl EventOut,
    store: &PartialRangeStore,
    tracked: &TrackedItems,
    emitted: &mut HashMap<String, DeliverySnapshot>,
) -> bool {
    let params = params_for(tracked.level(), EngineParams::default());
    let entries = tracked.snapshot();
    emitted.retain(|id, _| entries.iter().any(|(known, _)| known == id));
    for (id, meta) in &entries {
        let mut pass = Pass {
            store,
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
    params: &'a EngineParams,
    emitted: &'a mut HashMap<String, DeliverySnapshot>,
}

async fn emit_post(out: &impl EventOut, pass: &mut Pass<'_>, id: &str, meta: &VideoMeta) -> bool {
    let (ranges, stored_total) = match store_view(pass.store, id).await {
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
    let event = event_for(id, pass.emitted.get(id), current);
    pass.emitted.insert(id.to_owned(), current);
    match event {
        Some(event) => out.send(event),
        None => true,
    }
}

async fn store_view(
    store: &PartialRangeStore,
    id: &str,
) -> anyhow::Result<(Vec<ByteRange>, Option<u64>)> {
    let spans = store.present_ranges(id).await?;
    let ranges = spans
        .into_iter()
        .map(|span| ByteRange::new(span.start, span.end))
        .collect();
    Ok((ranges, store.total_len(id).await?))
}
