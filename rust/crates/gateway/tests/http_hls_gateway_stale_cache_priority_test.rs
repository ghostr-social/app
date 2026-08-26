mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, Response};
use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_origin::HlsOrigin;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_delivery::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_gateway::hls::sessions::HlsSessions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceExt as _;

#[tokio::test]
async fn validator_free_cached_backup_cannot_precede_a_live_primary() {
    let (_, backup) = HlsOrigin::start().await;
    let delivery = start_delivery("hls-stale-cache-priority");
    delivery.handle.update_focus(hls_focus(&backup));
    wait_ready(&delivery.segmented).await;
    let (primary, hits) = primary_origin().await;
    let sessions = HlsSessions::production();
    let session = sessions
        .acquire(vec![primary, backup])
        .await
        .expect("valid test fixture");
    let router = router_with_segmented_hls(sessions, media_client(), delivery.segmented);

    let request = Request::get(format!("/hls/{}/index.m3u8", session.as_str()))
        .body(Body::empty())
        .expect("valid test fixture");
    let response = router.oneshot(request).await.expect("valid test fixture");
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("valid test fixture");

    assert!(String::from_utf8_lossy(&body).contains("EXT-X-VERSION:9"));
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

async fn wait_ready(cache: &SegmentedCache) {
    let changed = cache.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        while cache.snapshot("stream").phase != SegmentedPhase::Ready {
            changed.notified().await;
        }
    })
    .await
    .expect("valid test fixture");
}

async fn primary_origin() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let hits = Arc::new(AtomicUsize::new(0));
    let observed = std::sync::Arc::clone(&hits);
    let app = Router::new().route(
        "/index.m3u8",
        get(move || {
            let observed = std::sync::Arc::clone(&observed);
            async move {
                observed.fetch_add(1, Ordering::SeqCst);
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
                    .body(Body::from(
                        "#EXTM3U\n#EXT-X-VERSION:9\n#EXTINF:4,\nprimary.m4s\n#EXT-X-ENDLIST\n",
                    ))
                    .expect("valid test fixture")
            }
        }),
    );
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    (format!("http://{address}/index.m3u8"), hits)
}
