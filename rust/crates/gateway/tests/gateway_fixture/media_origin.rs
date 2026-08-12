//! Range-capable loopback media used to observe real delivery body requests.

use std::time::Duration;
use tokio::net::TcpListener;

mod response;
mod server;
use server::OriginState;

pub struct MediaOrigin {
    base: String,
    state: OriginState,
    server: tokio::task::JoinHandle<()>,
}

impl MediaOrigin {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = OriginState::new();
        let app = server::router(state.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            base: format!("http://{address}"),
            state,
            server,
        }
    }

    pub fn url(&self, id: &str) -> String {
        format!("{}/{id}", self.base)
    }

    pub async fn assert_no_get(&self) {
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(
            self.gets().is_empty(),
            "body GET started before explicit focus"
        );
    }

    pub async fn wait_for_gets(&self, expected: &[&str]) {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if expected
                    .iter()
                    .all(|id| self.gets().iter().any(|hit| hit == id))
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("focused prefix body downloads");
    }

    fn gets(&self) -> Vec<String> {
        self.state.gets()
    }
}

impl Drop for MediaOrigin {
    fn drop(&mut self) {
        self.server.abort();
    }
}
