use axum::body::Body;
use axum::http::{Request, StatusCode};
use ghostr_gateway::http_gateway::configured_router;
use ghostr_media_model::native_models::new_native_downloads;
use tower::ServiceExt;

#[tokio::test]
async fn returns_not_found_for_an_unknown_video_identity() {
    let response = configured_router(new_native_downloads())
        .expect("router")
        .oneshot(
            Request::builder()
                .uri("/video.mp4?id=missing")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
