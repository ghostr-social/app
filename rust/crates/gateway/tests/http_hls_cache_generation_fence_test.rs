mod gateway_fixture;
use axum::body::{to_bytes, Body};
use axum::http::{header::RANGE, Request, StatusCode};
use core::time::Duration;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_origin::HlsOrigin;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_delivery::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_gateway::hls::sessions::HlsSessions;
use tower::ServiceExt as _;
#[tokio::test]
async fn ranged_hls_cache_generation_never_falls_through_to_origin() {
    let (origin, source) = HlsOrigin::start_cacheable().await;
    let delivery = start_delivery("hls-cache-generation-fence");
    delivery.handle.update_focus(hls_focus(&source));
    wait_ready(&delivery.segmented).await;
    let sessions = HlsSessions::production();
    let session = sessions
        .acquire(vec![source])
        .await
        .expect("valid test fixture");
    let router = router_with_segmented_hls(sessions, media_client(), delivery.segmented.clone());
    let root_path = format!("/hls/{}/index.m3u8", session.as_str());
    let root = router
        .clone()
        .oneshot(request(&root_path))
        .await
        .expect("valid test fixture");
    let manifest = to_bytes(root.into_body(), 4096)
        .await
        .expect("valid test fixture");
    let asset = asset_path(&manifest);
    let first = router
        .clone()
        .oneshot(ranged(&asset, "bytes=0-2"))
        .await
        .expect("valid test fixture");
    assert_eq!(
        to_bytes(first.into_body(), 3)
            .await
            .expect("valid test fixture"),
        "seg"
    );
    let second = router
        .clone()
        .oneshot(ranged(&asset, "bytes=3-5"))
        .await
        .expect("valid test fixture");
    assert_eq!(
        to_bytes(second.into_body(), 3)
            .await
            .expect("valid test fixture"),
        "men"
    );
    let hits = origin.hits();
    delivery.segmented.clear();
    let missing = router
        .oneshot(ranged(&asset, "bytes=0-2"))
        .await
        .expect("valid test fixture");
    assert_eq!(missing.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(origin.hits(), hits, "cache fence rejects before origin");
}

async fn wait_ready(cache: &SegmentedCache) {
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
    .expect("valid test fixture");
}

fn asset_path(manifest: &[u8]) -> String {
    String::from_utf8(manifest.to_vec())
        .expect("valid test fixture")
        .lines()
        .find(|line| line.starts_with("/hls/") && line.contains("/assets/"))
        .expect("valid test fixture")
        .to_owned()
}

fn ranged(uri: &str, range: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(RANGE, range)
        .body(Body::empty())
        .expect("valid test fixture")
}

fn request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("valid test fixture")
}
