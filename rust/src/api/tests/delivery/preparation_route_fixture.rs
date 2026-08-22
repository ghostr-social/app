use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use crate::api::playback_preparation_stream::{watch_preparation, PreparationOut};
use std::time::Duration;
use tokio::sync::mpsc;

mod authority;
use authority::RouteAuthority;

pub(super) const CURRENT_BYTES: &[u8; 16] = b"current-video-01";
pub(super) const NEXT_BYTES: &[u8; 16] = b"next-video-bytes";

pub(super) struct PreparationRouteFixture {
    plans: mpsc::UnboundedReceiver<FfiPlaybackPreparationPlan>,
    client: reqwest::Client,
    server: tokio::task::JoinHandle<()>,
}

impl PreparationRouteFixture {
    pub(super) async fn start() -> Self {
        let authority = RouteAuthority::seeded().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind gateway");
        let endpoint = listener.local_addr().expect("gateway address").to_string();
        let router = authority.router();
        let context = authority.context(endpoint);
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let (sender, plans) = mpsc::unbounded_channel();
        tokio::spawn(watch_preparation(ChannelOut(sender), context));
        Self {
            plans,
            client: reqwest::Client::builder().no_proxy().build().unwrap(),
            server,
        }
    }

    pub(super) async fn next_plan(&mut self) -> FfiPlaybackPreparationPlan {
        tokio::time::timeout(Duration::from_secs(1), self.plans.recv())
            .await
            .expect("preparation deadline")
            .expect("preparation plan")
    }

    pub(super) async fn get(&self, url: &str) -> (reqwest::StatusCode, Vec<u8>) {
        let response = self.client.get(url).send().await.expect("asset response");
        let status = response.status();
        let body = response.bytes().await.expect("asset bytes").to_vec();
        (status, body)
    }
}

impl Drop for PreparationRouteFixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

struct ChannelOut(mpsc::UnboundedSender<FfiPlaybackPreparationPlan>);

impl PreparationOut for ChannelOut {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool {
        self.0.send(plan).is_ok()
    }
}
