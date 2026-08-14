use axum::http::{Method, StatusCode};
use axum::routing::any;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Default)]
pub struct AttemptCounts {
    heads: AtomicUsize,
    bodies: AtomicUsize,
}

pub type Attempts = Arc<AttemptCounts>;

pub async fn serve() -> (String, Attempts) {
    let attempts = Arc::new(AttemptCounts::default());
    let observed = attempts.clone();
    let app = Router::new().route(
        "/video.mp4",
        any(move |method: Method| {
            if method == Method::HEAD {
                observed.heads.fetch_add(1, Ordering::SeqCst);
            } else {
                observed.bodies.fetch_add(1, Ordering::SeqCst);
            }
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
    attempts.heads.load(Ordering::SeqCst)
}

pub fn body_count(attempts: &Attempts) -> usize {
    attempts.bodies.load(Ordering::SeqCst)
}
