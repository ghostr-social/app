use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

mod response;
use response::{head_response, range_response};

#[derive(Clone)]
struct HostState {
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
}

pub struct SlowHost {
    base: String,
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
}

impl SlowHost {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("valid test fixture");
        let address = listener.local_addr().expect("valid test fixture");
        let started = Arc::new(Semaphore::new(0));
        let release = Arc::new(Semaphore::new(0));
        let state = HostState {
            started: Arc::clone(&started),
            release: Arc::clone(&release),
        };
        let app = Router::new().route("/{id}", get(media)).with_state(state);
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("valid test fixture");
        });
        Self {
            base: format!("http://{address}"),
            started,
            release,
        }
    }

    pub fn url(&self, id: &str) -> String {
        format!("{}/{id}", self.base)
    }

    pub fn localhost_url(&self, id: &str) -> String {
        self.url(id).replacen("127.0.0.1", "localhost", 1)
    }

    pub async fn wait_started(&self) {
        self.started
            .acquire()
            .await
            .expect("valid test fixture")
            .forget();
    }

    pub fn release(&self) {
        self.release.add_permits(2);
    }
}

async fn media(
    Path(_): Path<String>,
    State(state): State<HostState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if method == Method::HEAD {
        return head_response();
    }
    state.started.add_permits(1);
    state
        .release
        .acquire()
        .await
        .expect("valid test fixture")
        .forget();
    range_response(&headers)
}
