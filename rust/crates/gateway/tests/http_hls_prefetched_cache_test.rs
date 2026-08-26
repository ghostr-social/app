mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use core::time::Duration;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_origin::HlsOrigin;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_gateway::hls::sessions::{HlsSessionLimits, HlsSessions};
use tower::ServiceExt as _;

#[tokio::test]
async fn serves_every_prefetched_bootstrap_object_without_origin_refetch() {
    let (origin, source) = HlsOrigin::start_cacheable_master().await;
    let delivery = start_delivery("hls-prefetched-cache");
    delivery.handle.update_focus(hls_focus(&source));
    wait_ready(&delivery.segmented).await;
    let prefetched = vec!["root", "child", "init", "segment"];
    assert_eq!(origin.paths(), prefetched);
    assert_eq!(origin.hits(), 4);
    let sessions = sessions();
    let session = sessions
        .acquire(vec![source])
        .await
        .expect("valid test fixture");
    let router = router_with_segmented_hls(sessions, media_client(), delivery.segmented);

    let root = response_text(&router, &format!("/hls/{}/index.m3u8", session.as_str())).await;
    let child_path = root
        .lines()
        .find(|line| line.starts_with("/hls/") && line.contains("/manifests/"))
        .expect("valid test fixture")
        .to_owned();
    let child = response_text(&router, &child_path).await;
    let init_path = child
        .split("URI=\"")
        .nth(1)
        .expect("valid test fixture")
        .split('"')
        .next()
        .expect("valid test fixture");
    let segment_path = child
        .lines()
        .find(|line| line.starts_with("/hls/") && line.contains("/assets/"))
        .expect("valid test fixture");
    let init = router
        .clone()
        .oneshot(request(init_path))
        .await
        .expect("valid test fixture");
    let segment = router
        .oneshot(request(segment_path))
        .await
        .expect("valid test fixture");

    assert_eq!(
        to_bytes(init.into_body(), 64)
            .await
            .expect("valid test fixture"),
        "init"
    );
    assert_eq!(
        to_bytes(segment.into_body(), 64)
            .await
            .expect("valid test fixture"),
        "segment"
    );
    assert_eq!(origin.paths(), prefetched);
    assert_eq!(origin.hits(), 4);
}

async fn response_text(router: &axum::Router, path: &str) -> String {
    let response = router
        .clone()
        .oneshot(request(path))
        .await
        .expect("valid test fixture");
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("valid test fixture");
    String::from_utf8(body.to_vec()).expect("valid test fixture")
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
    .expect("valid test fixture");
}

fn sessions() -> HlsSessions {
    HlsSessions::new(
        HlsSessionLimits::new(2, Duration::from_secs(60), 8).expect("valid test fixture"),
    )
}

fn request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("valid test fixture")
}
