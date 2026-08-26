use super::asset_gated_origin::GatedOrigin;
use super::asset_sequence_origin::request_result;
use super::support::{asset_resource, state_with_timeouts};
use axum::body::to_bytes;
use core::time::Duration;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn overlapping_first_hls_ranges_share_one_generation_bootstrap() {
    let (source, origin) = GatedOrigin::start().await;
    let (state, session) = state_with_timeouts(source, timeouts()).await;
    let resource = asset_resource(&state, &session).await;
    let first = spawn(
        std::sync::Arc::clone(&state),
        session.clone(),
        resource.clone(),
        "bytes=0-3",
    );
    origin.wait_hits(1).await;
    let second = request_result(&state, &session, &resource, "bytes=4-7");
    tokio::pin!(second);

    assert!(
        tokio::time::timeout(Duration::from_millis(75), second.as_mut())
            .await
            .is_err()
    );
    origin.release_first();
    let first = tokio::time::timeout(Duration::from_secs(2), first)
        .await
        .expect("valid test fixture")
        .expect("valid test fixture")
        .expect("valid test fixture");
    let second = tokio::time::timeout(Duration::from_secs(2), second)
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert_eq!(
        to_bytes(first.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "abcd"
    );
    assert_eq!(
        to_bytes(second.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "efgh"
    );
    assert_eq!(origin.if_ranges(), [None, Some("\"v1\"".to_owned())]);
}

fn timeouts() -> HlsTransferTimeouts {
    HlsTransferTimeouts::new(
        Duration::from_millis(500),
        Duration::from_millis(500),
        Duration::from_secs(1),
    )
}

fn spawn(
    state: std::sync::Arc<crate::router::GatewayHttpState>,
    session: crate::hls::sessions::HlsSessionId,
    resource: String,
    range: &'static str,
) -> tokio::task::JoinHandle<Result<axum::response::Response, axum::http::StatusCode>> {
    tokio::spawn(async move { request_result(&state, &session, &resource, range).await })
}
