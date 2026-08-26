use axum::body::{Body, Bytes};
use axum::http::{header, StatusCode};
use axum::response::Response;
use core::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_stream::wrappers::ReceiverStream;

pub(super) fn head(total: usize) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, total)
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::empty())
        .expect("range-blind HEAD")
}

pub(super) fn split(
    bytes: Arc<Vec<u8>>,
    prefix_ready: Arc<Semaphore>,
    release: Arc<Semaphore>,
) -> Response {
    let total = bytes.len();
    let (sender, receiver) = tokio::sync::mpsc::channel::<Result<Bytes, Infallible>>(2);
    tokio::spawn(stream(bytes, prefix_ready, release, sender));
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, total)
        .body(Body::from_stream(ReceiverStream::new(receiver)))
        .expect("split range-blind response")
}

async fn stream(
    bytes: Arc<Vec<u8>>,
    prefix_ready: Arc<Semaphore>,
    release: Arc<Semaphore>,
    sender: tokio::sync::mpsc::Sender<Result<Bytes, Infallible>>,
) {
    let split = bytes.len().min(4_096);
    if sender
        .send(Ok(Bytes::copy_from_slice(&bytes[..split])))
        .await
        .is_err()
    {
        return;
    }
    prefix_ready.add_permits(1);
    let Ok(permit) = release.acquire().await else {
        return;
    };
    permit.forget();
    let _ = sender
        .send(Ok(Bytes::copy_from_slice(&bytes[split..])))
        .await;
}
