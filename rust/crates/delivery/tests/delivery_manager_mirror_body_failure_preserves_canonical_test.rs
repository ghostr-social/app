mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::gated_failure;
use delivery_fixture::items::{focus_now, seed_range};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::{DeliveryKind, EngineParams, PostId, VideoMeta};

const SHORT_206: &[u8] = b"HTTP/1.1 206 Partial Content\r\n\
Cache-Control: public, max-age=3600\r\nContent-Type: video/mp4\r\n\
Content-Length: 4\r\n\
Content-Range: bytes 0-3/16\r\n\
ETag: \"mirror\"\r\n\
Connection: close\r\n\r\nxy";

#[tokio::test]
async fn mirror_body_failure_preserves_the_canonical_prefix() {
    let mut primary = gated_failure::serve().await;
    let (mirror, request) = raw_http::spawn_raw_server(SHORT_206).await;
    let item = mirrored_item(primary.url(), &mirror);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness("mirror-body-preserves", options);
    seed_range(&harness.store, &item, 0, b"0123").await;

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    primary.wait_started().await;
    primary.release();
    tokio::time::timeout(Duration::from_secs(2), request)
        .await
        .expect("mirror request timeout")
        .expect("mirror request task");
    tokio::task::yield_now().await;
    delivery_fixture::wait::wait_not_servable(&harness.posts, "post").await;

    assert_eq!(
        harness
            .store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"0123".to_vec())
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(harness.root).ok();
}

fn mirrored_item(primary: &str, mirror: &str) -> FocusItem {
    FocusItem {
        post: PostId::new("post"),
        meta: VideoMeta {
            urls: vec![primary.to_owned(), mirror.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
    }
}
