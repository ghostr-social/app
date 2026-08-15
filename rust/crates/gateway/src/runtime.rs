#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use crate::debug::media::DebugMediaHttpClient;
use crate::delivery::start_progressive_delivery;
#[cfg(all(feature = "device-integration", debug_assertions))]
use crate::device_integration::DeviceIntegrationMediaHttpClient;
use crate::hls::playback::{HlsPlaybackGateway, NativeHlsPlaybackSession};
use crate::hls::sessions::HlsSessions;
use crate::progressive::capabilities::ProgressiveCapabilityId;
use crate::progressive::route::ProgressiveState;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_media_store::native_cache::prepare_native_cache_directory;
use ghostr_net::outbound_media_client::MediaHttpClient;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use log::warn;
use nostr_sdk::Client;
use std::{future::Future, io, path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, sync::watch};

pub struct GatewayConfiguration {
    pub cache_directory: PathBuf,
    pub relays: Vec<String>,
    pub max_parallel_downloads: usize,
    pub max_storage_bytes: u64,
    pub device_integration_origin: Option<String>,
}

pub struct GatewayRuntime {
    hls: HlsPlaybackGateway,
    delivery: DeliveryHandle,
    progressive: Arc<ProgressiveState>,
    segmented: SegmentedCache,
}

impl GatewayRuntime {
    /// Starts everything: endpoint, runtime, and the remaining discovery
    /// boot input (adaptive candidate demand). The shared Nostr
    /// client is supplied by the caller so this crate stays free of
    /// discovery.
    ///
    /// Contract on `client`: build it with `client_with_event_cache()`,
    /// never `Client::default()` — the shared client must retain verified
    /// events for session-scoped cache union and deduplication.
    pub async fn start(
        configuration: GatewayConfiguration,
        client: Arc<Client>,
    ) -> anyhow::Result<(String, Self, watch::Receiver<DiscoveryDemand>)> {
        let media = media_client(&configuration)?;
        start_with_media(configuration, client, media).await
    }

    #[cfg(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    ))]
    pub async fn start_debug(
        configuration: GatewayConfiguration,
        client: Arc<Client>,
    ) -> anyhow::Result<(String, Self, watch::Receiver<DiscoveryDemand>)> {
        let media = Arc::new(DebugMediaHttpClient::new()?);
        start_with_media(configuration, client, media).await
    }

    /// Control surface for focus, demand, and data-usage updates.
    pub fn delivery(&self) -> DeliveryHandle {
        self.delivery.clone()
    }

    /// Progressive plumbing (store/demand/posts) for the FFI layer.
    pub fn progressive(&self) -> Arc<ProgressiveState> {
        self.progressive.clone()
    }

    pub fn segmented(&self) -> SegmentedCache {
        self.segmented.clone()
    }

    pub async fn issue_progressive(&self, post: &str) -> ProgressiveCapabilityId {
        self.progressive.capabilities.issue(post).await
    }

    /// Applies the user's progressive-media budget without restarting
    /// the gateway; shrinking completes only after immediate eviction.
    pub async fn set_storage_budget(&self, budget: u64) -> anyhow::Result<()> {
        self.progressive.store.set_storage_budget(budget).await
    }

    pub async fn acquire_hls(
        &self,
        sources: Vec<String>,
    ) -> anyhow::Result<NativeHlsPlaybackSession> {
        self.hls.acquire(sources).await
    }

    pub async fn release_hls(&self, session_id: &str) -> bool {
        self.hls.release(session_id).await
    }
}

fn media_client(
    configuration: &GatewayConfiguration,
) -> anyhow::Result<Arc<dyn MediaHttpRequests>> {
    #[cfg(all(feature = "device-integration", debug_assertions))]
    if let Some(origin) = configuration.device_integration_origin.as_deref() {
        return Ok(Arc::new(DeviceIntegrationMediaHttpClient::new(origin)?));
    }
    anyhow::ensure!(
        configuration.device_integration_origin.is_none(),
        "device integration media is unavailable"
    );
    Ok(Arc::new(MediaHttpClient::public()?))
}

async fn start_with_media(
    configuration: GatewayConfiguration,
    client: Arc<Client>,
    media: Arc<dyn MediaHttpRequests>,
) -> anyhow::Result<(String, GatewayRuntime, watch::Receiver<DiscoveryDemand>)> {
    validate(&configuration)?;
    prepare_native_cache_directory(&configuration.cache_directory)?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let endpoint = address.to_string();
    let hls_sessions = HlsSessions::production();
    let (router, delivery, progressive, segmented, discovery_demand) =
        start_progressive_delivery(&configuration, hls_sessions.clone(), client, media).await?;
    spawn_http_server(listener, router);
    let hls = HlsPlaybackGateway::new(address, hls_sessions);
    let runtime = GatewayRuntime {
        hls,
        delivery,
        progressive,
        segmented,
    };
    Ok((endpoint, runtime, discovery_demand))
}

fn validate(configuration: &GatewayConfiguration) -> anyhow::Result<()> {
    anyhow::ensure!(
        configuration.max_parallel_downloads > 0,
        "download parallelism must be positive"
    );
    anyhow::ensure!(
        configuration.max_storage_bytes > 0,
        "native cache budget must be positive"
    );
    Ok(())
}

fn spawn_http_server(listener: TcpListener, app: axum::Router) {
    let server = async move { axum::serve(listener, app).await };
    tokio::spawn(supervise_http_server(server, report_http_server_failure));
}

pub async fn supervise_http_server<Server, Reporter>(server: Server, report: Reporter)
where
    Server: Future<Output = io::Result<()>>,
    Reporter: FnOnce(&io::Error),
{
    if let Err(error) = server.await {
        report(&error);
    }
}

pub fn report_http_server_failure(error: &io::Error) {
    warn!("Embedded video server failed: {error}");
}
