use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use core::convert::Infallible;
use core::time::Duration;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Notify};
use tokio_stream::wrappers::ReceiverStream;

mod request;

#[derive(Clone)]
struct Media {
    total: u64,
    header_delay: Duration,
    body_delay: Duration,
    release: Arc<Notify>,
}

pub async fn serve(total: u64, header_delay: Duration, body_delay: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind paced origin");
    let address = listener.local_addr().expect("paced origin address");
    let app = Router::new()
        .route("/video.mp4", get(response).head(response))
        .with_state(Media {
            total,
            header_delay,
            body_delay,
            release: Arc::new(Notify::new()),
        });
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve paced origin");
    });
    format!("http://{address}/video.mp4")
}

async fn response(State(media): State<Media>, method: Method, headers: HeaderMap) -> Response {
    if method == Method::HEAD {
        return builder(StatusCode::OK, media.total, None, Body::empty());
    }
    tokio::time::sleep(media.header_delay).await;
    let requested = request::parse(&headers, media.total);
    let (start, end) = requested.unwrap_or((0, media.total - 1));
    let length = end - start + 1;
    let status = requested.map_or(StatusCode::OK, |_| StatusCode::PARTIAL_CONTENT);
    let range = requested.map(|_| format!("bytes {start}-{end}/{}", media.total));
    builder(
        status,
        length,
        range,
        stalled_body(length, media.body_delay, media.release),
    )
}

fn stalled_body(length: u64, delay: Duration, release: Arc<Notify>) -> Body {
    let (sender, receiver) = mpsc::channel(1);
    tokio::spawn(async move {
        tokio::time::sleep(delay).await;
        let prefix_len = length.saturating_sub(1).min(64 * 1024);
        let prefix = vec![7; usize::try_from(prefix_len).expect("valid test fixture")];
        sender
            .send(Ok::<_, Infallible>(Bytes::from(prefix)))
            .await
            .ok();
        release.notified().await;
        let remaining = length.saturating_sub(prefix_len);
        let suffix = vec![8; usize::try_from(remaining).expect("valid test fixture")];
        sender
            .send(Ok::<_, Infallible>(Bytes::from(suffix)))
            .await
            .ok();
    });
    Body::from_stream(ReceiverStream::new(receiver))
}

fn builder(status: StatusCode, length: u64, range: Option<String>, body: Body) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"fixture-paced\"")
        .header(header::CONTENT_LENGTH, length);
    if let Some(range) = range {
        builder = builder.header(header::CONTENT_RANGE, range);
    }
    builder.body(body).expect("paced response")
}
