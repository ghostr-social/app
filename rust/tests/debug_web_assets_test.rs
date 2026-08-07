#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::{to_bytes, Body};
use axum::http::{header::CONTENT_TYPE, Request, StatusCode};
use support::progressive::progressive_harness;
use tower::ServiceExt;

async fn asset(path: &str) -> (StatusCode, String, String) {
    let harness = progressive_harness("ghostr-debug-assets");
    let request = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (
        status,
        content_type,
        String::from_utf8(body.to_vec()).expect("text"),
    )
}

#[tokio::test]
async fn debug_page_is_embedded_with_plain_web_assets() {
    let (status, content_type, html) = asset("/debug").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/html"));
    assert!(html.contains("<main"));
    assert!(html.contains("<video"));
    assert!(html.contains("<video id=\"player\" aria-labelledby=\"player-title\""));
    assert!(html.contains("controls playsinline muted autoplay"));
    assert!(html.contains("class=\"debug-layout\""));
    assert!(html.contains("class=\"phone-shell\""));
    assert!(html.contains("id=\"video-queue\""));
    assert!(html.contains("id=\"download-inspector\""));
    assert!(html.contains("id=\"nostr-relays\""));
    assert!(html.contains("Nostr discovery"));
    assert!(html.contains("<dialog id=\"network-modal\""));
    assert!(html.contains("aria-controls=\"network-modal\""));
    assert!(html.contains("id=\"video-url\""));
    assert!(html.contains("/debug/app.js"));
    assert!(html.contains("/debug/player_events.js"));
    assert!(html.contains("/debug/network_modal.js"));
    assert!(html.contains("/debug/nostr_feed.js"));
    assert!(html.contains("/debug/video_form.js"));

    let (status, _, trailing_html) = asset("/debug/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(trailing_html.contains("Video delivery debugger"));

    let (status, content_type, javascript) = asset("/debug/app.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/javascript"));
    assert!(javascript.contains("/debug/api/state"));
    assert!(javascript.contains("/debug/api/network"));
    assert!(javascript.contains("hydratePlayback(state)"));
    assert!(javascript.contains("state.nostr.current_id"));
    assert!(javascript.contains("startPlayback(video"));
    assert!(javascript.contains("play(video);"));

    let (status, content_type, player) = asset("/debug/player_events.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/javascript"));
    assert!(player.contains("player-error"));

    let (status, content_type, modal) = asset("/debug/network_modal.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/javascript"));
    assert!(modal.contains("showModal"));
    assert!(modal.contains("network-modal"));

    let (status, content_type, nostr) = asset("/debug/nostr_feed.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/javascript"));
    assert!(nostr.contains("/debug/api/focus"));
    assert!(nostr.contains("relay.status"));

    let (status, content_type, video_form) = asset("/debug/video_form.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/javascript"));
    assert!(video_form.contains("/debug/api/videos"));

    let (status, content_type, css) = asset("/debug/styles.css").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/css"));
    assert!(css.contains(":focus-visible"));
    assert!(css.contains(".player-empty[hidden]"));
}
