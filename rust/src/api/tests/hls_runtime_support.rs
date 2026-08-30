use super::hls_runtime_origin;
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission, FocusItem};
use ghostr_delivery::segmented::SegmentedSnapshot;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) async fn prepared() -> (GatewayRuntime, SegmentedSnapshot, PathBuf) {
    let source = hls_runtime_origin::start().await;
    let root = unique_root();
    let (_, runtime, _) = GatewayRuntime::start(
        configuration(root.clone(), &source),
        Arc::new(ghostr_discovery::cache::client_with_event_cache()),
    )
    .await
    .expect("gateway start");
    assert_eq!(
        runtime.delivery().update_focus(focus(&source)),
        FocusAdmission::Accepted
    );
    let cache = runtime.segmented();
    let changed = cache.notifier();
    let snapshot = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let snapshot = cache.snapshot("stream");
            if snapshot.authority.is_some() {
                return snapshot;
            }
            notified.await;
        }
    })
    .await
    .expect("prepared HLS authority");
    (runtime, snapshot, root)
}

fn configuration(cache_directory: PathBuf, source: &str) -> GatewayConfiguration {
    GatewayConfiguration {
        cache_directory,
        relays: Vec::new(),
        max_parallel_downloads: 2,
        max_storage_bytes: 1_048_576,
        network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::new(
            ghostr_engine::origin_model::NetworkClass::Wifi,
            1,
        ),
        device_integration_origin: Some(hls_runtime_origin::origin(source)),
    }
}

fn focus(source: &str) -> DeliveryFocus {
    let meta = crate::engine::VideoMeta {
        urls: vec![source.to_owned()],
        delivery: crate::engine::DeliveryKind::Hls,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(4_000),
    };
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: crate::engine::PostId::new("stream"),
            meta,
        }],
        0,
        0,
    )
}

fn unique_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "ghostr-api-hls-authority-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
