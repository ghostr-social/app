use axum::routing::get;
use axum::Router;
use core::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};

mod response;

use response::response;

pub(super) type RequestedRange = (usize, usize);
type BodyStarted = Arc<Mutex<Option<oneshot::Sender<RequestedRange>>>>;

#[derive(Clone)]
pub(super) struct OriginState {
    pub(super) bodies: Arc<AtomicUsize>,
    pub(super) requests: Arc<Mutex<Vec<RequestedRange>>>,
    pub(super) started: BodyStarted,
    pub(super) release: Arc<Notify>,
}

pub struct PressureOrigin {
    pub url: String,
    state: OriginState,
    started: Option<oneshot::Receiver<RequestedRange>>,
}

pub async fn serve() -> PressureOrigin {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind pressure origin");
    let address = listener.local_addr().expect("pressure origin address");
    let (signal, started) = oneshot::channel();
    let state = OriginState {
        bodies: Arc::new(AtomicUsize::new(0)),
        requests: Arc::new(Mutex::new(Vec::new())),
        started: Arc::new(Mutex::new(Some(signal))),
        release: Arc::new(Notify::new()),
    };
    let app = Router::new()
        .route("/video.mp4", get(response).head(response))
        .with_state(state.clone());
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve pressure origin");
    });
    PressureOrigin {
        url: format!("http://{address}/video.mp4"),
        state,
        started: Some(started),
    }
}

impl PressureOrigin {
    pub async fn wait_for_body(&mut self) -> RequestedRange {
        self.started
            .take()
            .expect("first body wait")
            .await
            .expect("body request")
    }

    pub fn release(&self) {
        self.state.release.notify_one();
    }

    pub fn requests(&self) -> Vec<RequestedRange> {
        self.state.requests.lock().expect("requests").clone()
    }
}
