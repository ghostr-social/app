//! Range-capable fixture with a caller-selected response Content-Type.

use axum::body::Body;
use axum::extract::State;
use axum::http::response::Builder;
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::Response;
use axum::routing::any;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
struct TypedMedia {
    bytes: Arc<Vec<u8>>,
    content_type: Option<HeaderValue>,
}

pub async fn serve(content_type: Option<&str>, bytes: Vec<u8>) -> String {
    let state = TypedMedia {
        bytes: Arc::new(bytes),
        content_type: content_type.map(|value| value.parse().expect("valid Content-Type")),
    };
    let app = Router::new()
        .route("/video.mp4", any(answer))
        .with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let address = listener.local_addr().expect("fixture address");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve fixture") });
    format!("http://{address}/video.mp4")
}

async fn answer(State(media): State<TypedMedia>, method: Method, headers: HeaderMap) -> Response {
    if method == Method::HEAD {
        return full(&media, Body::empty());
    }
    match requested_range(&headers, media.bytes.len() as u64) {
        Some((start, end)) => partial(&media, start, end),
        None => full(&media, Body::from(media.bytes.as_ref().clone())),
    }
}

fn full(media: &TypedMedia, body: Body) -> Response {
    build(media, StatusCode::OK, media.bytes.len())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .expect("full response")
}

fn partial(media: &TypedMedia, start: u64, end: u64) -> Response {
    let bytes = media.bytes[start as usize..=end as usize].to_vec();
    build(media, StatusCode::PARTIAL_CONTENT, bytes.len())
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{}", media.bytes.len()),
        )
        .body(Body::from(bytes))
        .expect("partial response")
}

fn build(media: &TypedMedia, status: StatusCode, length: usize) -> Builder {
    let builder = Response::builder()
        .status(status)
        .header(header::CONTENT_LENGTH, length);
    match &media.content_type {
        Some(value) => builder.header(header::CONTENT_TYPE, value),
        None => builder,
    }
}

fn requested_range(headers: &HeaderMap, len: u64) -> Option<(u64, u64)> {
    let value = headers.get(header::RANGE)?.to_str().ok()?;
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    let start = start.parse().ok()?;
    let end: u64 = end.parse().unwrap_or(len - 1);
    Some((start, end.min(len - 1)))
}
