use ghostr_delivery::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use std::time::Duration;

pub(super) async fn demand(runtime: &GatewayRuntime) -> DemandConsumer {
    let state = runtime.progressive();
    let binding = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if let Some(binding) = state.store.representation_binding("video").await {
                return binding;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("representation binding");
    let mut consumer = state.demand.consumer(PostId::new("video"), Some(binding));
    consumer.demand(ByteRange::new(0, 16));
    consumer
}

pub(super) fn configuration(root: std::path::PathBuf) -> GatewayConfiguration {
    GatewayConfiguration {
        cache_directory: root,
        relays: Vec::new(),
        max_parallel_downloads: 1,
        max_storage_bytes: 1_024,
        network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::unavailable(),
        device_integration_origin: None,
    }
}

pub(super) fn focus(url: &str) -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("video"),
            meta: VideoMeta {
                urls: vec![url.to_owned()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(32),
                duration_ms: Some(1_000),
            },
        }],
        0,
        0,
    )
}
