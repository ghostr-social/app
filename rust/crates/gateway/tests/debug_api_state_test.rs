#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::Body;
use axum::http::Request;
use gateway_fixture::debug_response::json_request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::{DeliveryKind, VideoMeta};
use serde_json::json;

#[tokio::test]
async fn debug_state_reports_downloaded_bytes_duration_and_source() {
    let harness = progressive_harness("ghostr-debug-state");
    let meta = VideoMeta {
        urls: vec!["https://relay.example/media/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(1_000),
        duration_ms: Some(120_000),
    };
    harness.bind_video_meta("clip", meta.clone()).await;
    harness
        .posts
        .replace([gateway_fixture::cache_video("clip", meta)]);
    harness
        .store
        .set_total_len("clip", 1_000)
        .await
        .expect("total");
    harness
        .store
        .write_range("clip", 0, &[7; 250])
        .await
        .expect("write");
    let request = Request::builder()
        .uri("/debug/api/state")
        .body(Body::empty())
        .expect("request");

    let state = json_request(&harness, request).await;

    assert_eq!(state["storage"]["used_bytes"], 250);
    assert_eq!(state["videos"][0]["id"], "clip");
    assert_eq!(state["videos"][0]["source_host"], "relay.example");
    assert_eq!(state["videos"][0]["downloaded_bytes"], 250);
    assert_eq!(state["videos"][0]["duration_ms"], 120_000);
    assert_eq!(state["videos"][0]["downloaded_duration_ms"], 30_000);
    assert_eq!(state["decisions"]["records"], json!([]));
    assert_eq!(
        state["evaluation"]["user_visible"]["swipe_to_first_frame"]["samples"],
        0
    );
    let playback_url = state["videos"][0]["playback_url"]
        .as_str()
        .expect("playback URL");
    assert!(playback_url.starts_with("/video.mp4?id=clip&cap="));
}
