use axum::http::StatusCode;
use axum::routing::any;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

pub type Attempts = Arc<AtomicUsize>;

pub async fn serve() -> (String, Attempts) {
    let attempts = Arc::new(AtomicUsize::new(0));
    let observed = attempts.clone();
    let app = Router::new().route(
        "/video.mp4",
        any(move || {
            observed.fetch_add(1, Ordering::SeqCst);
            async { StatusCode::INTERNAL_SERVER_ERROR }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind transient origin");
    let address = listener.local_addr().expect("transient origin address");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve transient origin")
    });
    (format!("http://{address}/video.mp4"), attempts)
}

pub fn count(attempts: &Attempts) -> usize {
    attempts.load(Ordering::SeqCst)
}
