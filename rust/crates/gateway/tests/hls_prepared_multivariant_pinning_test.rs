mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_origin::HlsOrigin;
use gateway_fixture::hls_prepared_session::wait_authority;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_gateway::hls::playback::HlsPlaybackRequest;
use ghostr_gateway::hls::sessions::HlsSessions;
use tower::ServiceExt as _;

#[tokio::test]
async fn prepared_multivariant_playback_serves_the_scheduler_selected_child() {
    let (origin, source) = HlsOrigin::start_cacheable_multivariant().await;
    let delivery = start_delivery("hls-prepared-multivariant-pinning");
    delivery.handle.update_focus(hls_focus(&source));
    let authority = wait_authority(&delivery.segmented, None).await;
    assert_eq!(origin.paths(), vec!["root", "child", "init", "segment"]);

    let sessions = HlsSessions::production();
    let request =
        HlsPlaybackRequest::new(authority, vec![source]).expect("valid prepared playback request");
    let session = sessions
        .acquire_prepared(&delivery.segmented, request)
        .await
        .expect("prepared playback session");
    let router = router_with_segmented_hls(sessions, media_client(), delivery.segmented);
    let root = response_text(&router, &format!("/hls/{}/index.m3u8", session.as_str())).await;

    assert!(
        !root.contains("#EXT-X-STREAM-INF"),
        "prepared playback exposed the multivariant master:\n{root}"
    );
    assert!(root.contains("#EXT-X-MAP"), "selected child was not served");
    assert!(
        root.contains("#EXTINF:4,"),
        "selected segment was not served"
    );
    assert_eq!(origin.paths(), vec!["root", "child", "init", "segment"]);
}

async fn response_text(router: &axum::Router, path: &str) -> String {
    let request = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("gateway request");
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("gateway response");
    let body = to_bytes(response.into_body(), 4_096)
        .await
        .expect("bounded manifest body");
    String::from_utf8(body.to_vec()).expect("UTF-8 manifest")
}
