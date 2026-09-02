use crate::api::delivery_events_stream::{watch_delivery, DeliveryWatchContext, EventOut};
use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::{command_channel, CommandReceiver, FocusGeneration};
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::PostId;
use tokio::sync::mpsc;

use super::plan;

struct ChannelOut(mpsc::UnboundedSender<FfiDeliveryEvent>);

impl EventOut for ChannelOut {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        self.0.send(event).is_ok()
    }
}

pub(super) struct ProgressiveEtaWatcher {
    commands: CommandReceiver,
    events: mpsc::UnboundedReceiver<FfiDeliveryEvent>,
}

impl ProgressiveEtaWatcher {
    pub(super) async fn start() -> Self {
        let store = temp_store("ghostr-api-progressive-eta");
        let meta = sized_meta(16, 2_000);
        bind_store(&store, "clip", &meta).await;
        let tracked = TrackedItems::new();
        let generation = FocusGeneration::try_new(7).expect("test generation");
        assert!(tracked.replace_focus(generation, vec![("clip".into(), meta.clone())]));
        let cache = CacheRegistry::new();
        cache.replace([CacheVideo {
            id: "clip".into(),
            meta,
            status: CacheStatus::Ready,
        }]);
        let (delivery, mut commands) = command_channel();
        commands.publish_causal_focused_plan(1, Some(PostId::new("clip")), 6, plan::plan(40));
        let context = DeliveryWatchContext::new(store, SegmentedCache::new(), tracked, cache)
            .with_delivery(delivery);
        let (sender, events) = mpsc::unbounded_channel();
        tokio::spawn(watch_delivery(ChannelOut(sender), context));
        Self { commands, events }
    }

    pub(super) fn publish_causal_plan(&mut self, eta_ms: u64) {
        self.commands.publish_causal_focused_plan(
            2,
            Some(PostId::new("clip")),
            7,
            plan::plan(eta_ms),
        );
    }

    pub(super) async fn expect_quiet(&mut self) {
        let quiet = tokio::time::timeout(Duration::from_millis(300), self.events.recv()).await;
        assert!(
            quiet.is_err(),
            "no event expected inside one ETA bucket: {quiet:?}"
        );
    }

    pub(super) async fn next(&mut self) -> FfiDeliveryEvent {
        tokio::time::timeout(Duration::from_secs(2), self.events.recv())
            .await
            .expect("delivery event deadline")
            .expect("open delivery stream")
    }
}
