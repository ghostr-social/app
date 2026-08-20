use crate::hls::sessions::{HlsSessionId, HlsSessions};
use crate::router::GatewayHttpState;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use std::time::Duration;

struct LocalClient(Client);

impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub(super) async fn state(source: String) -> (Arc<GatewayHttpState>, HlsSessionId) {
    let sessions = HlsSessions::production();
    let id = sessions.acquire(vec![source]).await.expect("session");
    let client = Client::builder().no_proxy().build().expect("client");
    let timeouts = HlsTransferTimeouts::new(
        Duration::from_millis(100),
        Duration::from_millis(10),
        Duration::from_millis(100),
    );
    let state = GatewayHttpState {
        client: Arc::new(LocalClient(client)),
        hls_sessions: sessions,
        segmented: SegmentedCache::new(),
        hls_timeouts: timeouts,
    };
    (Arc::new(state), id)
}
