use super::ActiveRequest;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode, Uri};
use axum::response::Response;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

type OriginState = (Arc<Vec<u8>>, mpsc::Sender<ActiveRequest>);

pub async fn response(
    State((bytes, requests)): State<OriginState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let total = bytes.len() as u64;
    if method == Method::HEAD {
        return reply(StatusCode::OK, total, None, Body::empty());
    }
    let requested = requested_range(&headers, total);
    let range = requested.clone().unwrap_or(0..total);
    let length = range.end - range.start;
    let content = requested.map(|span| format!("bytes {}-{}/{}", span.start, span.end - 1, total));
    let (body, stream) = mpsc::channel(8);
    requests
        .send(ActiveRequest {
            path: uri.path().to_owned(),
            range,
            body,
            bytes,
        })
        .await
        .ok();
    reply(
        request_status(&content),
        length,
        content,
        Body::from_stream(ReceiverStream::new(stream)),
    )
}

fn request_status(range: &Option<String>) -> StatusCode {
    range
        .as_ref()
        .map_or(StatusCode::OK, |_| StatusCode::PARTIAL_CONTENT)
}

fn reply(status: StatusCode, length: u64, range: Option<String>, body: Body) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"fixture-concurrency\"")
        .header(header::CONTENT_LENGTH, length);
    if let Some(range) = range {
        builder = builder.header(header::CONTENT_RANGE, range);
    }
    builder.body(body).expect("origin response")
}

fn requested_range(headers: &HeaderMap, total: u64) -> Option<core::ops::Range<u64>> {
    let value = headers.get(header::RANGE)?.to_str().ok()?;
    let (start, end) = value.trim_start_matches("bytes=").split_once('-')?;
    let start = start.parse().ok()?;
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1);
    Some(start..end + 1)
}
