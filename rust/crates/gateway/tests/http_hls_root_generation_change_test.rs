mod gateway_fixture;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Request, Response};
use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_gateway::hls::sessions::HlsSessions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceExt as _;

#[tokio::test]
async fn changed_root_generation_invalidates_and_reprepares_its_bootstrap_cohort() {
    let (source, root_hits) = origin().await;
    let delivery = start_delivery("hls-root-generation-change");
    delivery.handle.update_focus(hls_focus(&source));
    wait_ready(&delivery).await;
    let old_init = source.replace("index.m3u8", "init-v1.mp4");
    assert!(delivery.segmented.object(&old_init).is_some());
    let sessions = HlsSessions::production();
    let session = sessions
        .acquire(vec![source])
        .await
        .expect("valid test fixture");
    let router = router_with_segmented_hls(sessions, media_client(), delivery.segmented.clone());
    let request = Request::get(format!("/hls/{}/index.m3u8", session.as_str()))
        .body(Body::empty())
        .expect("valid test fixture");

    router.oneshot(request).await.expect("valid test fixture");

    assert_eq!(root_hits.load(Ordering::SeqCst), 2);
    assert!(delivery.segmented.object(&old_init).is_none());
    assert_eq!(
        delivery.segmented.snapshot("stream").phase,
        SegmentedPhase::Queued
    );
    wait_ready(&delivery).await;
}

async fn wait_ready(delivery: &gateway_fixture::delivery::DeliveryFixture) {
    let changed = delivery.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        while delivery.segmented.snapshot("stream").phase != SegmentedPhase::Ready {
            changed.notified().await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "recovery stalled: {:?}; plan: {:?}",
            delivery.segmented.snapshot("stream"),
            delivery.handle.latest_plan()
        )
    });
}

async fn origin() -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let app = Router::new()
        .route("/index.m3u8", get(root))
        .route(
            "/init-v1.mp4",
            get(|| async { cacheable("init", "init-v1") }),
        )
        .route(
            "/segment-v1.m4s",
            get(|| async { cacheable("segment", "segment-v1") }),
        )
        .with_state(std::sync::Arc::clone(&hits));
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    (format!("http://{address}/index.m3u8"), hits)
}

async fn root(State(hits): State<Arc<AtomicUsize>>) -> Response<Body> {
    let version = hits.fetch_add(1, Ordering::SeqCst) + 1;
    Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "max-age=0")
        .header(header::ETAG, format!("\"root-v{version}\""))
        .body(Body::from(
            "#EXTM3U\n#EXT-X-MAP:URI=\"init-v1.mp4\"\n#EXTINF:4,\nsegment-v1.m4s\n#EXT-X-ENDLIST\n",
        ))
        .expect("valid test fixture")
}

fn cacheable(body: &'static str, etag: &'static str) -> Response<Body> {
    Response::builder()
        .header(header::CACHE_CONTROL, "max-age=60")
        .header(header::ETAG, format!("\"{etag}\""))
        .body(Body::from(body))
        .expect("valid test fixture")
}
