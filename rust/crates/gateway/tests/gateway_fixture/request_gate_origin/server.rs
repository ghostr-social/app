use super::{ActiveBody, VideoState};
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, Response, StatusCode};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub(super) async fn video(
    State((total, bodies)): State<VideoState>,
    method: Method,
    headers: HeaderMap,
) -> Response<Body> {
    if method == Method::HEAD {
        return reply(StatusCode::OK, total, None, Body::empty());
    }
    let (start, end) = requested(&headers, total);
    let (body, stream) = mpsc::channel(2);
    bodies
        .send(ActiveBody {
            length: (end - start) as usize,
            body,
        })
        .await
        .ok();
    reply(
        StatusCode::PARTIAL_CONTENT,
        end - start,
        Some(format!("bytes {start}-{}/{total}", end - 1)),
        Body::from_stream(ReceiverStream::new(stream)),
    )
}

pub(super) async fn manifest(
    State(hits): State<mpsc::Sender<()>>,
) -> ([(&'static str, &'static str); 1], &'static str) {
    hits.send(()).await.ok();
    (
        [("content-type", "application/vnd.apple.mpegurl")],
        "#EXTM3U\n",
    )
}

fn reply(status: StatusCode, length: u64, range: Option<String>, body: Body) -> Response<Body> {
    let mut response = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, length)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"gate\"");
    if let Some(range) = range {
        response = response.header(header::CONTENT_RANGE, range);
    }
    response.body(body).unwrap()
}

fn requested(headers: &HeaderMap, total: u64) -> (u64, u64) {
    let value = headers[header::RANGE]
        .to_str()
        .unwrap()
        .trim_start_matches("bytes=");
    let (start, end) = value.split_once('-').unwrap();
    let start = start.parse().unwrap();
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1) + 1;
    (start, end)
}
