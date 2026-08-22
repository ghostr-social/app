use super::asset_origin::{serve_asset, serve_optional_asset};
use super::support::{asset_resource, state};
use crate::hls::routes::asset;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::RANGE;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

pub(super) struct AssetExchange {
    pub result: Result<Response<Body>, StatusCode>,
    pub requests: Vec<String>,
}

pub(super) async fn exchange(response: Vec<u8>, ranges: &[&str]) -> AssetExchange {
    run(serve_asset(response).await, ranges).await
}

pub(super) async fn optional_exchange(response: Vec<u8>, ranges: &[&str]) -> AssetExchange {
    run(serve_optional_asset(response).await, ranges).await
}

async fn run(origin: (String, JoinHandle<Vec<String>>), ranges: &[&str]) -> AssetExchange {
    let (source, server) = origin;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;
    let mut headers = HeaderMap::new();
    for value in ranges {
        headers.append(RANGE, HeaderValue::from_str(value).expect("Range"));
    }
    let result = asset(
        State(state),
        Path((session.as_str().to_owned(), resource)),
        headers,
    )
    .await;
    let requests = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("asset request")
        .expect("origin server");
    AssetExchange { result, requests }
}

pub(super) fn range_values(request: &str) -> Vec<&str> {
    request
        .lines()
        .filter_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("range: ")
                .map(|_| line)
        })
        .filter_map(|line| line.split_once(':').map(|(_, value)| value.trim()))
        .collect()
}

pub(super) async fn observed_body(response: Response<Body>) -> (usize, bool) {
    let mut stream = response.into_body().into_data_stream();
    let mut bytes = 0;
    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => bytes += chunk.len(),
            Err(_) => return (bytes, true),
        }
    }
    (bytes, false)
}
