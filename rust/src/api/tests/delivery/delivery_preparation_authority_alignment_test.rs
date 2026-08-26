use crate::api::delivery_events_stream::{watch_delivery, DeliveryWatchContext, EventOut};
use crate::api::delivery_types::{FfiDeliveryEvent, FfiPlaybackPreparationPlan};
use crate::api::playback_preparation_stream::{
    watch_preparation, PreparationContext, PreparationOut,
};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::command_channel;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::PostId;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use tokio::sync::mpsc;

struct DeliveryOut(mpsc::UnboundedSender<FfiDeliveryEvent>);

impl EventOut for DeliveryOut {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        self.0.send(event).is_ok()
    }
}

struct PreparationOutbox(mpsc::UnboundedSender<FfiPlaybackPreparationPlan>);

impl PreparationOut for PreparationOutbox {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool {
        self.0.send(plan).is_ok()
    }
}

#[tokio::test]
async fn delivery_and_preparation_share_exact_asset_authority() {
    let store = temp_store("ghostr-delivery-preparation-authority");
    let tracked = TrackedItems::new();
    let meta = sized_meta(16, 2_000);
    bind_store(&store, "clip", &meta).await;
    store
        .set_total_len("clip", 16)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range("clip", 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    tracked.insert("clip".to_owned(), meta.clone());
    let cache = CacheRegistry::new();
    cache.replace([CacheVideo {
        id: "clip".to_owned(),
        meta,
        status: CacheStatus::Complete,
    }]);
    let capabilities = ProgressiveCapabilities::production();
    let (handle, mut commands) = command_channel();
    commands.publish_focused_plan(7, Some(PostId::new("clip")), Default::default());
    let (delivery_tx, mut delivery_rx) = mpsc::unbounded_channel();
    let delivery = DeliveryWatchContext::new(
        std::sync::Arc::clone(&store),
        SegmentedCache::new(),
        tracked.clone(),
        cache.clone(),
    )
    .with_capabilities(capabilities.clone());
    tokio::spawn(watch_delivery(DeliveryOut(delivery_tx), delivery));
    let (preparation_tx, mut preparation_rx) = mpsc::unbounded_channel();
    tokio::spawn(watch_preparation(
        PreparationOutbox(preparation_tx),
        PreparationContext {
            endpoint: "127.0.0.1:8080".to_owned(),
            store,
            capabilities,
            delivery: handle,
            tracked,
            cache,
        },
    ));

    let event = receive(&mut delivery_rx).await;
    let plan = receive(&mut preparation_rx).await;
    let asset = plan.current.expect("current asset");
    assert_eq!(event.representation_id, Some(asset.representation_id));
    assert_eq!(event.asset_id, Some(asset.asset_id));
}

async fn receive<T>(receiver: &mut mpsc::UnboundedReceiver<T>) -> T {
    tokio::time::timeout(Duration::from_secs(2), receiver.recv())
        .await
        .expect("event deadline")
        .expect("open stream")
}
