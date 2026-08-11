use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::convert::Infallible;
use std::ops::Range;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

type OriginState = (u64, mpsc::Sender<ActiveRequest>);

pub struct ControlledOrigin {
    pub url: String,
    requests: mpsc::Receiver<ActiveRequest>,
}

impl ControlledOrigin {
    pub async fn serve(total: u64) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind concurrency origin");
        let address = listener.local_addr().expect("concurrency origin address");
        let (requests, observed) = mpsc::channel(8);
        let app = Router::new()
            .route("/video.mp4", get(response).head(response))
            .with_state((total, requests));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            url: format!("http://{address}/video.mp4"),
            requests: observed,
        }
    }

    pub async fn next(&mut self) -> ActiveRequest {
        self.requests.recv().await.expect("origin stays available")
    }
}

pub struct ActiveRequest {
    pub range: Range<u64>,
    body: mpsc::Sender<Result<Bytes, Infallible>>,
}

impl ActiveRequest {
    pub async fn send_byte(&self) -> bool {
        self.body.send(Ok(Bytes::from_static(&[7]))).await.is_ok()
    }
}

async fn response(
    State((total, requests)): State<OriginState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if method == Method::HEAD {
        return reply(StatusCode::OK, total, None, Body::empty());
    }
    let range = requested(&headers, total);
    let length = range.end - range.start;
    let content_range = format!("bytes {}-{}/{}", range.start, range.end - 1, total);
    let (body, stream) = mpsc::channel(8);
    requests.send(ActiveRequest { range, body }).await.ok();
    reply(
        StatusCode::PARTIAL_CONTENT,
        length,
        Some(content_range),
        Body::from_stream(ReceiverStream::new(stream)),
    )
}

fn reply(status: StatusCode, length: u64, range: Option<String>, body: Body) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, length);
    if let Some(range) = range {
        builder = builder.header(header::CONTENT_RANGE, range);
    }
    builder.body(body).expect("concurrency response")
}

fn requested(headers: &HeaderMap, total: u64) -> Range<u64> {
    let value = headers[header::RANGE].to_str().expect("range header");
    let (start, end) = value.trim_start_matches("bytes=").split_once('-').unwrap();
    let start = start.parse().unwrap();
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1);
    start..end + 1
}
