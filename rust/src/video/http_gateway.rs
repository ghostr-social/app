use crate::video::native_models::NativeDownloads;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
struct VideoProxyState {
    client: Client,
    downloads: NativeDownloads,
}

#[derive(Deserialize)]
struct VideoQuery {
    id: String,
}

pub fn configured_router(downloads: NativeDownloads) -> Router {
    let state = Arc::new(VideoProxyState {
        client: Client::new(),
        downloads,
    });
    Router::new()
        .route("/status", get(gateway_status))
        .route("/video.mp4", get(stream_video))
        .with_state(state)
}

async fn gateway_status() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn stream_video(
    State(state): State<Arc<VideoProxyState>>,
    Query(query): Query<VideoQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let url = video_url(&state.downloads, &query.id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let upstream = upstream_request(&state.client, url, &headers)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    proxy_response(upstream)
}

async fn video_url(downloads: &NativeDownloads, id: &str) -> Option<String> {
    downloads
        .lock()
        .await
        .get(id)
        .map(|video| video.url.clone())
}

fn upstream_request(client: &Client, url: String, headers: &HeaderMap) -> reqwest::RequestBuilder {
    let request = client.get(url);
    match headers.get(RANGE) {
        Some(value) => request.header(RANGE, value),
        None => request,
    }
}

fn proxy_response(upstream: reqwest::Response) -> Result<Response, StatusCode> {
    let status = upstream.status();
    let headers = upstream.headers().clone();
    let mut response = Response::builder().status(status);
    for name in [CONTENT_TYPE, CONTENT_LENGTH, CONTENT_RANGE, ACCEPT_RANGES] {
        if let Some(value) = headers.get(&name) {
            response = response.header(name, value);
        }
    }
    response
        .body(Body::from_stream(upstream.bytes_stream()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
