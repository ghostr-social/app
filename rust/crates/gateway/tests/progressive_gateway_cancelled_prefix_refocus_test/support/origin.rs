use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod active_request;
mod server;
pub use active_request::ActiveRequest;

pub struct ControlledOrigin {
    base_url: String,
    requests: mpsc::Receiver<ActiveRequest>,
    task: tokio::task::JoinHandle<()>,
}

impl ControlledOrigin {
    pub async fn serve(bytes: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind origin");
        let address = listener.local_addr().expect("origin address");
        let (requests, observed) = mpsc::channel(8);
        let bytes = Arc::new(bytes);
        let app = Router::new()
            .route("/{video}", get(server::response).head(server::response))
            .with_state((bytes, requests));
        let task =
            tokio::spawn(async move { axum::serve(listener, app).await.expect("serve origin") });
        Self {
            base_url: format!("http://{address}"),
            requests: observed,
            task,
        }
    }

    pub fn url_for(&self, id: &str) -> String {
        format!("{}/{id}.mp4", self.base_url)
    }

    pub async fn next(&mut self) -> ActiveRequest {
        self.requests
            .recv()
            .await
            .expect("origin remains available")
    }
}

impl Drop for ControlledOrigin {
    fn drop(&mut self) {
        self.task.abort();
    }
}
