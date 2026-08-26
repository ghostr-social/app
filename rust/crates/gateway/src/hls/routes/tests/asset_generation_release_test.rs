use super::asset_gated_origin::GatedOrigin;
use super::asset_sequence_origin::request_result;
use super::support::{asset_resource, state};
use axum::http::StatusCode;

#[tokio::test]
async fn releasing_hls_session_while_asset_headers_wait_prevents_admission() {
    let (source, origin) = GatedOrigin::start().await;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;
    let task_state = std::sync::Arc::clone(&state);
    let task_session = session.clone();
    let pending = tokio::spawn(async move {
        request_result(&task_state, &task_session, &resource, "bytes=0-3").await
    });
    origin.wait_hits(1).await;

    assert!(state.hls_sessions.release(&session).await);
    origin.release_first();
    let result = tokio::time::timeout(core::time::Duration::from_secs(2), pending)
        .await
        .expect("released request completion")
        .expect("asset task");
    assert_eq!(
        result.expect_err("released session"),
        StatusCode::BAD_GATEWAY
    );
}
