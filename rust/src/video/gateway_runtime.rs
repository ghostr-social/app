use crate::engine::inventory_controller::Mode;
use crate::engine::{DataUsageLevel, EngineParams};
use crate::video::delivery_events::DeliveryHandle;
use crate::video::delivery_manager::{
    start_delivery_manager_with_modes, DeliveryManagerConfig, DeliveryTuning,
};
use crate::video::event_identity::VIDEO_KINDS;
pub use crate::video::event_index::MAX_NATIVE_INVENTORY_ITEMS;
use crate::video::event_index::{spawn_event_identity_indexer, NativeVideoIndex};
use crate::video::hls_playback_gateway::{HlsPlaybackGateway, NativeHlsPlaybackSession};
use crate::video::hls_sessions::HlsSessions;
use crate::video::http_gateway::configured_router_with_progressive;
use crate::video::native_cache::prepare_native_cache_directory;
use crate::video::native_models::{new_native_downloads, NativeVideoDownload};
use crate::video::outbound_media_client::MediaHttpClient;
use crate::video::partial_range_store::PartialRangeStore;
use crate::video::playback_demand::demand_channel;
use crate::video::progressive_posts::ServablePosts;
use crate::video::progressive_route::{ProgressiveState, ProgressiveTiming};
use log::warn;
use nostr_sdk::{Client, Filter, Kind};
use std::{future::Future, io, path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, sync::watch, sync::Mutex};

type DeliveryParts = (axum::Router, DeliveryHandle, Arc<ProgressiveState>, watch::Receiver<Mode>);

pub struct GatewayConfiguration {
    pub cache_directory: PathBuf,
    pub relays: Vec<String>,
    pub max_parallel_downloads: usize,
    pub max_storage_bytes: u64,
}

pub struct GatewayRuntime {
    hls: HlsPlaybackGateway,
    videos: NativeVideoIndex,
    delivery: DeliveryHandle,
    progressive: Arc<ProgressiveState>,
}

impl GatewayRuntime {
    /// Starts everything: endpoint, runtime, and the discovery boot
    /// inputs (shared Nostr client + inventory-mode watch, plan §5.4).
    pub async fn start(
        configuration: GatewayConfiguration,
    ) -> anyhow::Result<(String, Self, Arc<Client>, watch::Receiver<Mode>)> {
        validate(&configuration)?;
        prepare_native_cache_directory(&configuration.cache_directory)?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let endpoint = address.to_string();
        let client = Arc::new(Client::default());
        let videos = NativeVideoIndex::new(MAX_NATIVE_INVENTORY_ITEMS);
        spawn_event_identity_indexer(client.clone(), videos.clone());
        configure_client(&client, &configuration.relays).await;
        let hls_sessions = HlsSessions::production();
        let (router, delivery, progressive, modes) =
            start_progressive_delivery(&configuration, hls_sessions.clone())?;
        spawn_http_server(listener, router);
        let hls = HlsPlaybackGateway::new(address, hls_sessions);
        let runtime = Self { hls, videos, delivery, progressive };
        Ok((endpoint, runtime, client, modes))
    }

    /// Indexed feed metadata for the Dart fallback, from the index.
    pub async fn discovered_videos(&self) -> Vec<NativeVideoDownload> {
        self.videos
            .ordered_videos()
            .await
            .into_iter()
            .map(|item| NativeVideoDownload::new(item.inventory_id, item.video, item.identity))
            .collect()
    }

    /// Control surface for focus, demand, and data-usage updates.
    pub fn delivery(&self) -> DeliveryHandle {
        self.delivery.clone()
    }

    /// Progressive plumbing (store/demand/posts) for the FFI layer.
    pub fn progressive(&self) -> Arc<ProgressiveState> {
        self.progressive.clone()
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

async fn configure_client(client: &Client, relays: &[String]) {
    for relay in relays {
        if let Err(error) = client.add_relay(relay).await {
            warn!("Nostr relay {relay} was rejected: {error}");
        }
    }
    client.connect().await;
    if let Err(error) = client
        .subscribe(vec![video_filter(), deletion_filter()], None)
        .await
    {
        warn!("Nostr video subscription failed: {error}");
    }
}

pub fn video_filter() -> Filter {
    let kinds = VIDEO_KINDS.into_iter().map(Kind::Custom);
    Filter::new().kinds(kinds).limit(MAX_NATIVE_INVENTORY_ITEMS)
}

pub fn deletion_filter() -> Filter {
    Filter::new()
        .kind(Kind::EventDeletion)
        .limit(MAX_NATIVE_INVENTORY_ITEMS)
}

/// Progressive delivery: the router serves `/video.mp4` from the partial
/// store; the manager's mode watch feeds the discovery control loop.
fn start_progressive_delivery(
    configuration: &GatewayConfiguration,
    hls_sessions: HlsSessions,
) -> anyhow::Result<DeliveryParts> {
    let store = Arc::new(PartialRangeStore::new(
        configuration.cache_directory.join("progressive"),
        Arc::new(Mutex::new(0)),
    ));
    let posts = ServablePosts::new();
    let (demand_sender, demand) = demand_channel();
    let client = MediaHttpClient::public()?;
    let progressive = Arc::new(ProgressiveState {
        store: store.clone(),
        demand: demand_sender,
        posts: posts.clone(),
        timing: ProgressiveTiming::default(),
    });
    let router = configured_router_with_progressive(
        new_native_downloads(), hls_sessions, client.clone(), progressive.clone(),
    );
    let config = delivery_config(configuration, store, client, posts);
    let (delivery, modes) = start_delivery_manager_with_modes(config, demand);
    Ok((router, delivery, progressive, modes))
}

fn delivery_config(
    configuration: &GatewayConfiguration,
    store: Arc<PartialRangeStore>,
    client: MediaHttpClient,
    posts: ServablePosts,
) -> DeliveryManagerConfig {
    let params = EngineParams {
        balanced_concurrency: configuration.max_parallel_downloads,
        ..EngineParams::default()
    };
    DeliveryManagerConfig {
        store,
        client,
        posts,
        stats_path: configuration.cache_directory.join("host_stats.json"),
        params,
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
    }
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
