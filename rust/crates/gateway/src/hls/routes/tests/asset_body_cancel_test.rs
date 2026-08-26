use super::asset_origin::serve_stalled_asset;
use super::support::{asset_resource, state_with_timeouts};
use crate::hls::routes::asset;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use core::time::Duration;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn dropping_the_player_body_closes_the_origin_transfer_promptly() {
    let (source, server) = serve_stalled_asset().await;
    let deadlines = HlsTransferTimeouts::new(
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let (state, session) = state_with_timeouts(source, deadlines).await;
    let resource = asset_resource(&state, &session).await;
    let response = asset(
        State(state),
        Path((session.as_str().to_owned(), resource)),
        HeaderMap::new(),
    )
    .await
    .expect("asset headers");

    drop(response);
    tokio::time::timeout(Duration::from_millis(250), server)
        .await
        .expect("origin closes with player body")
        .expect("origin server");
}
