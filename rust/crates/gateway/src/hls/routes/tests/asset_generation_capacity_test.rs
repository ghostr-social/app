use super::asset_capacity_origin::serve;
use super::asset_sequence_origin::{request, request_error};
use super::support::{asset_resources, state_with_sessions};
use crate::hls::sessions::{HlsSessionLimits, HlsSessions};
use axum::body::to_bytes;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn hls_generation_capacity_refuses_new_asset_without_evicting_bound_asset() {
    let (source, server) = serve().await;
    let limits = HlsSessionLimits::new(1, Duration::from_secs(60), 1).unwrap();
    let sessions = HlsSessions::new(limits);
    let (state, session) =
        state_with_sessions(source, HlsTransferTimeouts::default(), sessions).await;
    let resources = asset_resources(&state, &session).await;

    let first = request(&state, &session, &resources[0], "bytes=0-3").await;
    assert_eq!(to_bytes(first.into_body(), 4).await.unwrap(), "abcd");
    assert_eq!(
        request_error(&state, &session, &resources[1], "bytes=0-3").await,
        502
    );
    let continued = request(&state, &session, &resources[0], "bytes=4-7").await;
    assert_eq!(to_bytes(continued.into_body(), 4).await.unwrap(), "efgh");

    let requests = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests
        .iter()
        .all(|request| request.contains("/first.m4s ")));
}
