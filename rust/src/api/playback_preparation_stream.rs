//! Atomic current and upcoming playback assets selected by WARP.

use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use crate::api::runtime::registry;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::future::Future;
use std::sync::Arc;

pub(crate) mod projection;

pub(crate) trait PreparationOut: Send + 'static {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool;
}

impl PreparationOut for StreamSink<FfiPlaybackPreparationPlan> {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool {
        self.add(plan).is_ok()
    }
}

pub(crate) struct PreparationContext {
    pub(crate) endpoint: String,
    pub(crate) store: Arc<PartialRangeStore>,
    pub(crate) capabilities: ProgressiveCapabilities,
    pub(crate) delivery: DeliveryHandle,
    pub(crate) tracked: TrackedItems,
    pub(crate) cache: CacheRegistry,
}

/// Streams one atomic current-plus-upcoming window; cancellation drops the watcher.
#[frb]
pub async fn ffi_playback_preparation_events(
    sink: StreamSink<FfiPlaybackPreparationPlan>,
) -> anyhow::Result<()> {
    let engine = registry::engine()?;
    let progressive = engine.gateway.progressive();
    let context = PreparationContext {
        endpoint: engine.endpoint.clone(),
        store: progressive.store.clone(),
        capabilities: progressive.capabilities.clone(),
        delivery: engine.gateway.delivery(),
        tracked: engine.tracked.clone(),
        cache: progressive.cache.clone(),
    };
    tokio::spawn(watch_preparation(sink, context));
    Ok(())
}

pub(crate) async fn watch_preparation(out: impl PreparationOut, context: PreparationContext) {
    let plan_changed = context.delivery.plan_notifier();
    let store_changed = context.store.change_notifier();
    let tracked_changed = context.tracked.notifier();
    let cache_changed = context.cache.notifier();
    let mut previous = None;
    loop {
        let plan_wake = plan_changed.notified();
        let store_wake = store_changed.notified();
        let tracked_wake = tracked_changed.notified();
        let cache_wake = cache_changed.notified();
        tokio::pin!(plan_wake, store_wake, tracked_wake, cache_wake);
        plan_wake.as_mut().enable();
        store_wake.as_mut().enable();
        tracked_wake.as_mut().enable();
        cache_wake.as_mut().enable();
        let current = projection::project(&context).await;
        if current != previous {
            let Some(plan) = current.clone() else {
                wait_for_change(plan_wake, store_wake, tracked_wake, cache_wake).await;
                continue;
            };
            if !out.send(plan) {
                return;
            }
            previous = current;
        }
        wait_for_change(plan_wake, store_wake, tracked_wake, cache_wake).await;
    }
}

async fn wait_for_change<P, S, T, C>(plan: P, store: S, tracked: T, cache: C)
where
    P: Future<Output = ()>,
    S: Future<Output = ()>,
    T: Future<Output = ()>,
    C: Future<Output = ()>,
{
    tokio::select! { _ = plan => {}, _ = store => {}, _ = tracked => {}, _ = cache => {} }
}
