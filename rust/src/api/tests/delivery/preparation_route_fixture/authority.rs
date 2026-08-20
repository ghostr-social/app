use super::{CURRENT_BYTES, NEXT_BYTES};
use crate::api::playback_preparation_stream::PreparationContext;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use ghostr_delivery::debug::feed::DebugFeed;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::{command_channel, DeliveryHandle};
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_gateway::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_gateway::router::configured_router_with_progressive;
use ghostr_net::outbound_media_client::MediaHttpClient;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;

mod plan;

pub(super) struct RouteAuthority {
    store: Arc<PartialRangeStore>,
    tracked: TrackedItems,
    cache: CacheRegistry,
    capabilities: ProgressiveCapabilities,
    delivery: DeliveryHandle,
}

impl RouteAuthority {
    pub(super) async fn seeded() -> Self {
        let store = temp_store("ghostr-preparation-route");
        let tracked = TrackedItems::new();
        seed(&store, &tracked, "current", CURRENT_BYTES).await;
        seed(&store, &tracked, "next", NEXT_BYTES).await;
        let cache = CacheRegistry::new();
        cache.replace([cached("current"), cached("next")]);
        let (delivery, mut commands) = command_channel();
        plan::publish(&mut commands, &store).await;
        Self {
            store,
            tracked,
            cache,
            capabilities: ProgressiveCapabilities::production(),
            delivery,
        }
    }

    pub(super) fn context(&self, endpoint: String) -> PreparationContext {
        PreparationContext {
            endpoint,
            store: self.store.clone(),
            capabilities: self.capabilities.clone(),
            delivery: self.delivery.clone(),
            tracked: self.tracked.clone(),
            cache: self.cache.clone(),
        }
    }

    pub(super) fn router(&self) -> axum::Router {
        let (demand, _) = demand_channel();
        let state = Arc::new(ProgressiveState {
            store: self.store.clone(),
            demand,
            cache: self.cache.clone(),
            network: NetworkThrottle::new(),
            timing: ProgressiveTiming::default(),
            capabilities: self.capabilities.clone(),
            #[cfg(all(
                feature = "video-debug-web",
                debug_assertions,
                not(any(target_os = "android", target_os = "ios"))
            ))]
            debug_feed: DebugFeed::new(self.delivery.clone(), Vec::new()),
        });
        configured_router_with_progressive(
            HlsSessions::production(),
            Arc::new(MediaHttpClient::public().unwrap()),
            state,
        )
    }
}

async fn seed(store: &PartialRangeStore, tracked: &TrackedItems, id: &str, bytes: &[u8]) {
    let meta = sized_meta(bytes.len() as u64, 2_000);
    bind_store(store, id, &meta).await;
    store.set_total_len(id, bytes.len() as u64).await.unwrap();
    store.write_range(id, 0, bytes).await.unwrap();
    tracked.insert(id.to_owned(), meta);
}

fn cached(id: &str) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: sized_meta(16, 2_000),
        status: CacheStatus::Complete,
    }
}
