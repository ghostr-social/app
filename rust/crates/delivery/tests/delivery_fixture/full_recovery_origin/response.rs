use super::{ObservedRequest, OriginState, PARALLEL_BYTES, PROBE_BYTES, TRIAL_BYTES};
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode, Uri};
use axum::response::Response;
use core::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub(super) async fn answer(
    State(requests): State<OriginState>,
    uri: Uri,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let total = total(uri.path());
    let range = text_header(&headers, header::RANGE);
    let encoding = text_header(&headers, header::ACCEPT_ENCODING);
    let (body, stream) = mpsc::channel(2);
    requests
        .send(ObservedRequest {
            method: method.clone(),
            path: uri.path().to_owned(),
            range: range.clone(),
            encoding,
            body: (method != Method::HEAD).then_some(body),
        })
        .await
        .ok();
    response(total, range.as_deref(), &method, stream)
}

fn response(
    total: usize,
    range: Option<&str>,
    method: &Method,
    stream: mpsc::Receiver<Result<Bytes, Infallible>>,
) -> Response {
    let requested = range.map(|value| bounds(value, total));
    let partial = requested.is_some();
    let status = if partial {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"recovery\"");
    if let Some((start, end)) = requested {
        let length = end - start + 1;
        builder = builder.header(header::CONTENT_LENGTH, length).header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        );
    }
    let body = (method == Method::HEAD)
        .then(Body::empty)
        .unwrap_or_else(|| Body::from_stream(ReceiverStream::new(stream)));
    builder.body(body).expect("recovery response")
}

fn bounds(value: &str, total: usize) -> (usize, usize) {
    let value = value.strip_prefix("bytes=").expect("range unit");
    let (start, end) = value.split_once('-').expect("range bounds");
    let start = start.parse().expect("range start");
    let end = end.parse().unwrap_or(total - 1).min(total - 1);
    (start, end)
}

fn text_header(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn total(path: &str) -> usize {
    match path {
        "/probe.mp4" => PROBE_BYTES,
        "/parallel.mp4" => PARALLEL_BYTES,
        _ => TRIAL_BYTES,
    }
}
