mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_origin::HlsOrigin;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_gateway::hls::sessions::{HlsSessionLimits, HlsSessions};
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test]
async fn serves_prefetched_manifest_and_segment_without_origin_refetch() {
    let (origin, source) = HlsOrigin::start().await;
    let delivery = start_delivery("hls-prefetched-cache");
    delivery.handle.update_focus(hls_focus(&source));
    wait_ready(&delivery.segmented).await;
    let prefetched_hits = origin.hits();
    let sessions = sessions();
    let session = sessions.acquire(vec![source]).await.unwrap();
    let router = router_with_segmented_hls(sessions, media_client(), delivery.segmented);

    let root = router
        .clone()
        .oneshot(request(&format!("/hls/{}/index.m3u8", session.as_str())))
        .await
        .unwrap();
    let manifest = to_bytes(root.into_body(), 4096).await.unwrap();
    let segment_path = String::from_utf8(manifest.to_vec())
        .unwrap()
        .lines()
        .find(|line| line.starts_with("/hls/") && line.contains("/assets/"))
        .unwrap()
        .to_owned();
    let segment = router.oneshot(request(&segment_path)).await.unwrap();

    assert_eq!(to_bytes(segment.into_body(), 64).await.unwrap(), "segment");
    assert_eq!(origin.hits(), prefetched_hits);
}

async fn wait_ready(cache: &ghostr_delivery::segmented::SegmentedCache) {
    let changed = cache.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if cache.snapshot("stream").phase == SegmentedPhase::Ready {
                return;
            }
            notified.await;
        }
    })
    .await
    .unwrap();
}

fn sessions() -> HlsSessions {
    HlsSessions::new(HlsSessionLimits::new(2, Duration::from_secs(60), 8).unwrap())
}

fn request(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}
