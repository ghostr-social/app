use super::identity_origin::{assert_identity_request, coded_manifest, manifest_then_coded_asset};
use super::support::{asset_resource, state};
use crate::hls::routes::{asset, root_manifest};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};

#[tokio::test]
async fn coded_manifest_is_rejected_after_requesting_identity() {
    let (source, server) = coded_manifest().await;
    let (state, session) = state(source).await;

    let result = root_manifest(State(state), Path(session.as_str().to_owned())).await;

    assert_eq!(result.expect_err("coded manifest"), StatusCode::BAD_GATEWAY);
    let requests = server.await.expect("origin server");
    assert_identity_request(&requests[0]);
}

#[tokio::test]
async fn coded_uncached_asset_is_never_proxied_to_the_player() {
    let (source, server) = manifest_then_coded_asset().await;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;

    let result = asset(
        State(state),
        Path((session.as_str().to_owned(), resource)),
        HeaderMap::new(),
    )
    .await;

    assert_eq!(result.expect_err("coded asset"), StatusCode::BAD_GATEWAY);
    let requests = server.await.expect("origin server");
    assert_identity_request(&requests[0]);
    assert_identity_request(&requests[1]);
}
