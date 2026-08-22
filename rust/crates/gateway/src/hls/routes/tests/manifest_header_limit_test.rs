use super::origin::oversized_manifest_headers;
use super::support::state;
use crate::hls::routes::root_manifest;
use axum::extract::{Path, State};
use axum::http::StatusCode;

#[tokio::test]
async fn uncached_manifest_rejects_oversized_origin_headers() {
    let (source, server) = oversized_manifest_headers().await;
    let (state, session) = state(source).await;

    let result = root_manifest(State(state), Path(session.as_str().to_owned())).await;

    assert_eq!(
        result.expect_err("oversized headers"),
        StatusCode::BAD_GATEWAY
    );
    server.await.expect("server");
}
