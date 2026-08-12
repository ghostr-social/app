#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use gateway_fixture::progressive::progressive_harness;
use serde_json::Value;
use tower::ServiceExt;

const REGISTRATIONS: usize = 65;

#[tokio::test]
async fn manual_focus_is_exact_and_bounded() {
    let harness = progressive_harness("debug-manual-focus");
    let mut ids = Vec::new();
    for index in 0..REGISTRATIONS {
        ids.push(add_video(&harness.router, index).await);
    }

    assert_eq!(
        select(&harness.router, &ids[0]).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        select(&harness.router, &ids[1]).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        select(&harness.router, "missing").await,
        StatusCode::NOT_FOUND
    );
    std::fs::remove_dir_all(harness.root).ok();
}

async fn add_video(router: &Router, index: usize) -> String {
    let body = format!(r#"{{"url":"https://cdn.example/{index}.mp4"}}"#);
    let request = Request::post("/debug/api/videos")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");
    let response = router.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), 256).await.expect("body");
    let response: Value = serde_json::from_slice(&body).expect("JSON");
    response["id"].as_str().expect("video id").to_owned()
}

async fn select(router: &Router, id: &str) -> StatusCode {
    let body = format!(r#"{{"id":"{id}"}}"#);
    let request = Request::put("/debug/api/focus")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");
    router
        .clone()
        .oneshot(request)
        .await
        .expect("response")
        .status()
}
