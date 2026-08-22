use super::origin::partial_manifest;
use super::support::state;
use crate::hls::routes::root_manifest;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use std::time::Duration;

#[tokio::test]
async fn partial_manifest_is_never_rewritten_as_complete() {
    let (source, server) = partial_manifest().await;
    let (state, session) = state(source).await;

    let result = root_manifest(State(state), Path(session.as_str().to_owned())).await;
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("origin request")
        .expect("server");

    assert_eq!(
        result.expect_err("partial manifest"),
        StatusCode::BAD_GATEWAY
    );
}
