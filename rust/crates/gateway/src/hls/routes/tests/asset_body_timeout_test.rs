use super::origin::manifest_then_stalled_asset;
use super::support::state;
use crate::hls::routes::{asset, root_manifest};
use axum::body::to_bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

#[tokio::test]
async fn uncached_asset_body_has_an_idle_deadline() {
    let (source, server) = manifest_then_stalled_asset().await;
    let (state, session) = state(source).await;
    let root = root_manifest(State(state.clone()), Path(session.as_str().to_owned()))
        .await
        .expect("root manifest");
    let body = to_bytes(root.into_body(), 4096).await.expect("root body");
    let manifest = String::from_utf8(body.to_vec()).expect("manifest");
    let resource = manifest
        .split("/assets/")
        .nth(1)
        .and_then(|suffix| suffix.lines().next())
        .expect("asset");

    let response = asset(
        State(state),
        Path((session.as_str().to_owned(), resource.to_owned())),
        HeaderMap::new(),
    )
    .await
    .expect("asset headers");

    assert!(to_bytes(response.into_body(), 4096).await.is_err());
    server.abort();
}
