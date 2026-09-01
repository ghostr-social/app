//! R→D delivery events (plan §2 row 5).
//!
//! A watcher task derives per-post readiness and progress from the partial store.
//! The manager's writes wake it through the store's change notifier, so every
//! manager state change surfaces without polling.

use crate::api::delivery::causal_eta::plan_notifier;
use crate::api::delivery::emit::{DeliverySnapshot, Pass};
use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::runtime::registry;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::engine::budget::params_for;
use crate::engine::EngineParams;
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
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

#[derive(Clone)]
pub(crate) struct DeliveryWatchContext {
    store: Arc<PartialRangeStore>,
    segmented: SegmentedCache,
    tracked: TrackedItems,
    cache: CacheRegistry,
    capabilities: ProgressiveCapabilities,
    delivery: Option<DeliveryHandle>,
}

impl DeliveryWatchContext {
    pub(super) fn new(
        store: Arc<PartialRangeStore>,
        segmented: SegmentedCache,
        tracked: TrackedItems,
        cache: CacheRegistry,
    ) -> Self {
        Self {
            store,
            segmented,
            tracked,
            cache,
            capabilities: ProgressiveCapabilities::production(),
            delivery: None,
        }
    }

    /// Lets the watcher read causal plans for readiness ETAs.
    pub(crate) fn with_delivery(mut self, delivery: DeliveryHandle) -> Self {
        self.delivery = Some(delivery);
        self
    }

    pub(super) fn with_capabilities(mut self, capabilities: ProgressiveCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Subscribes to per-post delivery events. Each subscription runs its
/// own watcher and first reports a readiness baseline for every
/// watched post; the stream ends when the Dart side cancels it.
#[frb]
pub async fn ffi_delivery_events(sink: StreamSink<FfiDeliveryEvent>) -> anyhow::Result<()> {
    let engine = registry::engine()?;
    let progressive = engine.gateway.progressive();
    let context = DeliveryWatchContext::new(
        progressive.store.clone(),
        engine.gateway.segmented(),
        engine.tracked.clone(),
        progressive.cache.clone(),
    )
    .with_capabilities(progressive.capabilities.clone())
    .with_delivery(engine.gateway.delivery());
    tokio::spawn(watch_delivery(sink, context));
    Ok(())
}

pub(crate) async fn watch_delivery(out: impl EventOut, context: DeliveryWatchContext) {
    let store_changed = context.store.change_notifier();
    let items_changed = context.tracked.notifier();
    let segmented_changed = context.segmented.notifier();
    let cache_changed = context.cache.notifier();
    let plan_changed = plan_notifier(context.delivery.as_ref());
    let mut emitted: HashMap<String, DeliverySnapshot> = HashMap::new();
    loop {
        let store_wake = store_changed.notified();
        let items_wake = items_changed.notified();
        let segmented_wake = segmented_changed.notified();
        let cache_wake = cache_changed.notified();
        let plan_wake = plan_changed.notified();
        tokio::pin!(
            store_wake,
            items_wake,
            segmented_wake,
            cache_wake,
            plan_wake
        );
        store_wake.as_mut().enable();
        items_wake.as_mut().enable();
        segmented_wake.as_mut().enable();
        cache_wake.as_mut().enable();
        plan_wake.as_mut().enable();
        if !emit_pass(&out, &context, &mut emitted).await {
            return;
        }
        tokio::select! {
            () = store_wake => {},
            () = items_wake => {},
            () = segmented_wake => {},
            () = cache_wake => {},
            () = plan_wake => {},
        }
    }
}

async fn emit_pass(
    out: &impl EventOut,
    context: &DeliveryWatchContext,
    emitted: &mut HashMap<String, DeliverySnapshot>,
) -> bool {
    let params = params_for(context.tracked.level(), EngineParams::default());
    let entries = context.tracked.snapshot();
    let focus_generation = context.tracked.focus_generation();
    emitted.retain(|id, _| entries.iter().any(|(known, _)| known == id));
    for (id, meta) in &entries {
        let mut pass = Pass {
            store: &context.store,
            segmented: &context.segmented,
            cache: &context.cache,
            capabilities: &context.capabilities,
            params: &params,
            delivery: context.delivery.as_ref(),
            focus_generation,
            emitted,
        };
        if !crate::api::delivery::emit::emit_post(out, &mut pass, id, meta).await {
            return false;
        }
    }
    true
}
