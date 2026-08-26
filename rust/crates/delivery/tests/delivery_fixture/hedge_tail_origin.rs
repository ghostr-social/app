use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Response};
use axum::routing::get;
use axum::Router;
use core::future::pending;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Notify;

mod response;

#[derive(Clone)]
struct OriginState {
    started: Arc<Notify>,
    total: u64,
    range_bytes: u64,
    stalled: bool,
}

pub struct HedgeTailOrigins {
    pub primary_url: String,
    pub alternate_url: String,
    primary_started: Arc<Notify>,
    alternate_started: Arc<Notify>,
}

impl HedgeTailOrigins {
    pub async fn serve(total: u64, range_bytes: u64) -> Self {
        let primary_started = Arc::new(Notify::new());
        let alternate_started = Arc::new(Notify::new());
        let primary_url = spawn_origin(OriginState {
            started: Arc::clone(&primary_started),
            total,
            range_bytes,
            stalled: true,
        })
        .await;
        let alternate_url = spawn_origin(OriginState {
            started: Arc::clone(&alternate_started),
            total,
            range_bytes,
            stalled: false,
        })
        .await;
        Self {
            primary_url,
            alternate_url,
            primary_started,
            alternate_started,
        }
    }

    pub async fn wait_primary(&self) {
        self.primary_started.notified().await;
    }

    pub async fn wait_alternate(&self) {
        self.alternate_started.notified().await;
    }
}

async fn spawn_origin(state: OriginState) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let app = Router::new()
        .route("/video.mp4", get(reply).head(reply))
        .with_state(state);
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    format!("http://{address}/video.mp4")
}

async fn reply(State(state): State<OriginState>, method: Method) -> Response<Body> {
    if method == Method::HEAD {
        return response::head(state.total);
    }
    state.started.notify_one();
    if state.stalled {
        return pending().await;
    }
    response::partial(state.total, state.range_bytes)
}
