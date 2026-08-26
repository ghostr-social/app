use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::router::GatewayRouterResources;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpClient;
use std::sync::Arc;

pub(super) fn router_resources() -> GatewayRouterResources {
    let requests = MediaRequestExecutor::new(
        Arc::new(MediaHttpClient::public().expect("test fixture precondition must hold")),
        MediaRequestLimits::try_new(3, 3).expect("test fixture precondition must hold"),
    );
    GatewayRouterResources::new(HlsSessions::production(), requests)
}
