use crate::hls::routes::root_manifest;
use crate::hls::sessions::{HlsSessionId, HlsSessions};
use crate::router::GatewayHttpState;
use axum::body::to_bytes;
use axum::extract::{Path, State};
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
    let timeouts = HlsTransferTimeouts::new(
        Duration::from_millis(100),
        Duration::from_millis(10),
        Duration::from_millis(100),
    );
    state_with_timeouts(source, timeouts).await
}

pub(super) async fn state_with_timeouts(
    source: String,
    timeouts: HlsTransferTimeouts,
) -> (Arc<GatewayHttpState>, HlsSessionId) {
    let sessions = HlsSessions::production();
    state_with_sessions(source, timeouts, sessions).await
}

pub(super) async fn state_with_sessions(
    source: String,
    timeouts: HlsTransferTimeouts,
    sessions: HlsSessions,
) -> (Arc<GatewayHttpState>, HlsSessionId) {
    let id = sessions.acquire(vec![source]).await.expect("session");
    let client = Client::builder().no_proxy().build().expect("client");
    let state = GatewayHttpState {
        client: Arc::new(LocalClient(client)),
        hls_sessions: sessions,
        segmented: SegmentedCache::new(),
        hls_timeouts: timeouts,
    };
    (Arc::new(state), id)
}

pub(super) async fn asset_resource(
    state: &Arc<GatewayHttpState>,
    session: &HlsSessionId,
) -> String {
    asset_resources(state, session).await.remove(0)
}

pub(super) async fn asset_resources(
    state: &Arc<GatewayHttpState>,
    session: &HlsSessionId,
) -> Vec<String> {
    let response = root_manifest(State(state.clone()), Path(session.as_str().to_owned()))
        .await
        .expect("root manifest");
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("manifest body");
    String::from_utf8(body.to_vec())
        .expect("manifest")
        .lines()
        .filter_map(|line| {
            line.split_once("/assets/")
                .map(|(_, token)| token.to_owned())
        })
        .collect()
}
