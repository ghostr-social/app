use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

pub const TOTAL: u64 = 16 * 1024 * 1024;
pub struct Origin { pub url: String, pub whole_requests: Arc<AtomicUsize> }

pub async fn serve() -> Origin {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("origin");
    let url = format!("http://{}/video.mp4", listener.local_addr().expect("address"));
    let whole_requests = Arc::new(AtomicUsize::new(0));
    let app = Router::new().route("/video.mp4", get(answer)).with_state(Arc::clone(&whole_requests));
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve origin") });
    Origin { url, whole_requests }
}

async fn answer(State(whole): State<Arc<AtomicUsize>>, method: Method, headers: HeaderMap) -> Response {
    let mut response = Response::builder().header("content-type", "video/mp4")
        .header("etag", "W/\"realistic-weak-validator\"").header("cache-control", "public, max-age=3600");
    if let Some(range) = headers.get("range") {
        let (start, end) = range.to_str().expect("range").trim_start_matches("bytes=").split_once('-').expect("bounds");
        let start: u64 = start.parse().expect("start");
        let end: u64 = end.parse().expect("end");
        let length = end.min(TOTAL - 1) - start + 1;
        return response.status(206).header("content-range", format!("bytes {start}-{}/{TOTAL}", start + length - 1))
            .header("content-length", length).body(Body::from(vec![7; length as usize])).expect("partial response");
    }
    response = response.header("content-length", TOTAL);
    if method == Method::HEAD { return response.body(Body::empty()).expect("head"); }
    whole.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        for _ in 0..TOTAL / (16 * 1024) {
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
            if tx.send(Ok::<_, std::io::Error>(Bytes::from(vec![7; 16 * 1024]))).await.is_err() { break; }
        }
    });
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    response.body(Body::from_stream(stream)).expect("whole response")
}

pub async fn wait_at(store: &PartialRangeStore, offset: u64) {
    loop {
        if store.read_range("current", offset..offset + 4).await.expect("read").as_deref() == Some(&[7; 4]) { return; }
        tokio::time::sleep(core::time::Duration::from_millis(10)).await;
    }
}
