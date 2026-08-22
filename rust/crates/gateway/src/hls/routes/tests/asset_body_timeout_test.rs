use super::origin::manifest_then_stalled_asset;
use super::support::{asset_resource, state};
use crate::hls::routes::asset;
use axum::body::to_bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

#[tokio::test]
async fn uncached_asset_body_has_an_idle_deadline() {
    let (source, server) = manifest_then_stalled_asset().await;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;

    let response = asset(
        State(state),
        Path((session.as_str().to_owned(), resource)),
        HeaderMap::new(),
    )
    .await
    .expect("asset headers");

    assert!(to_bytes(response.into_body(), 4096).await.is_err());
    server.abort();
}
