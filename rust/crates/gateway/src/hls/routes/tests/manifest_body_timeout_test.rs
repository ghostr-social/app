use super::origin::stalled_manifest_body;
use super::support::state;
use crate::hls::routes::root_manifest;
use axum::extract::{Path, State};
use axum::http::StatusCode;

#[tokio::test]
async fn uncached_manifest_body_has_an_idle_deadline() {
    let (source, server) = stalled_manifest_body().await;
    let (state, session) = state(source).await;

    let result = root_manifest(State(state), Path(session.as_str().to_owned())).await;

    assert_eq!(result.expect_err("stalled body"), StatusCode::BAD_GATEWAY);
    server.abort();
}
