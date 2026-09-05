mod support;

use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission, FocusItem};
use ghostr_gateway::hls::playback::HlsPlaybackRequest;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use rust_lib_ghostr::engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use support::{fixtures::temp_directory, hls_prepared_origin};

#[tokio::test]
async fn runtime_rejects_a_stale_prepared_hls_authority() {
    let source = hls_prepared_origin::start().await;
    let directory = temp_directory("ghostr-hls-runtime-authority");
    let (_endpoint, runtime, _modes) = GatewayRuntime::start(
        configuration(directory.clone(), &source),
        Arc::new(ghostr_discovery::cache::client_with_event_cache()),
    )
    .await
    .expect("gateway start");
    assert_eq!(
        runtime.delivery().update_focus(focus(&source)),
        FocusAdmission::Accepted
    );
    let cache = runtime.segmented();
    let authority = wait_authority(&cache).await;
    let request =
        HlsPlaybackRequest::new(authority.clone(), vec![source.clone()]).expect("prepared request");
    let session = runtime
        .acquire_hls(request)
        .await
        .expect("prepared session");
    assert_eq!(session.authority.as_ref(), Some(&authority));

    let generation = cache.object(&source).expect("prepared root").generation();
    assert!(cache.invalidate_generation(&source, generation));
    let stale = HlsPlaybackRequest::new(authority, vec![source]).expect("stale request");
    assert!(runtime.acquire_hls(stale).await.is_err());
    assert!(runtime.release_hls(session.id.as_str()).await);
    std::fs::remove_dir_all(directory).ok();
}

fn configuration(cache_directory: std::path::PathBuf, source: &str) -> GatewayConfiguration {
    GatewayConfiguration {
        cache_directory,
        relays: Vec::new(),
        max_parallel_downloads: 2,
        max_storage_bytes: 1_048_576,
        internet_data_limit: ghostr_net::internet_allowance::InternetDataLimit::Unlimited,
        network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::new(
            rust_lib_ghostr::engine::origin_model::NetworkClass::Wifi,
            1,
        ),
        device_integration_origin: Some(hls_prepared_origin::origin(source)),
    }
}

fn focus(source: &str) -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec![source.to_owned()],
                delivery: DeliveryKind::Hls,
                sha256: None,
                size_bytes: None,
                duration_ms: Some(4_000),
            },
        }],
        0,
        0,
    )
}

async fn wait_authority(
    cache: &ghostr_delivery::segmented::SegmentedCache,
) -> ghostr_delivery::segmented::HlsPreparedAssetAuthority {
    let changed = cache.notifier();
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(authority) = cache.snapshot("stream").authority {
                return authority;
            }
            notified.await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "prepared authority deadline: {:?}",
            cache.snapshot("stream")
        )
    })
}
