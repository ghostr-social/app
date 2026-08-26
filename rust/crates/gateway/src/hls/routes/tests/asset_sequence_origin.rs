use super::asset_origin::{accept, write_manifest};
use crate::hls::routes::asset;
use axum::extract::{Path, State};
use axum::http::header::RANGE;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use core::time::Duration;
use tokio::io::AsyncWriteExt as _;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::timeout;

pub(super) async fn serve(responses: Vec<&'static [u8]>) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let source = format!(
        "http://{}/index.m3u8",
        listener.local_addr().expect("valid test fixture")
    );
    let task = tokio::spawn(async move {
        let (mut root, _) = accept(&listener).await;
        write_manifest(&mut root).await;
        drop(root);
        let mut requests = Vec::new();
        for response in responses {
            let (mut asset, request) = accept(&listener).await;
            asset.write_all(response).await.expect("asset response");
            requests.push(request);
        }
        if let Ok((asset, request)) = timeout(Duration::from_millis(150), accept(&listener)).await {
            requests.push(request);
            drop(asset);
        }
        requests
    });
    (source, task)
}

pub(super) fn header_values<'a>(request: &'a str, name: &str) -> Vec<&'a str> {
    request
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(found, _)| found.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim())
        .collect()
}

pub(super) async fn request(
    state: &std::sync::Arc<crate::router::GatewayHttpState>,
    session: &crate::hls::sessions::HlsSessionId,
    resource: &str,
    range: &str,
) -> Response<axum::body::Body> {
    request_result(state, session, resource, range)
        .await
        .expect("asset response")
}

pub(super) async fn request_error(
    state: &std::sync::Arc<crate::router::GatewayHttpState>,
    session: &crate::hls::sessions::HlsSessionId,
    resource: &str,
    range: &str,
) -> StatusCode {
    request_result(state, session, resource, range)
        .await
        .expect_err("asset rejection")
}

pub(super) async fn request_result(
    state: &std::sync::Arc<crate::router::GatewayHttpState>,
    session: &crate::hls::sessions::HlsSessionId,
    resource: &str,
    range: &str,
) -> Result<Response<axum::body::Body>, StatusCode> {
    asset(
        State(std::sync::Arc::clone(state)),
        Path((session.as_str().to_owned(), resource.to_owned())),
        headers(range),
    )
    .await
}

fn headers(range: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(RANGE, HeaderValue::from_str(range).expect("Range"));
    headers
}
