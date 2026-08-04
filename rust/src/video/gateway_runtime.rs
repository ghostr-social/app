use crate::discovery::event_cache::client_with_event_cache;
use crate::engine::inventory_controller::Mode;
use crate::video::delivery_events::DeliveryHandle;
use crate::video::event_identity::VIDEO_KINDS;
pub use crate::video::event_index::MAX_NATIVE_INVENTORY_ITEMS;
use crate::video::event_index::{spawn_event_identity_indexer, NativeVideoIndex};
use crate::video::gateway_delivery::start_progressive_delivery;
use crate::video::hls_playback_gateway::{HlsPlaybackGateway, NativeHlsPlaybackSession};
use crate::video::hls_sessions::HlsSessions;
use crate::video::native_cache::prepare_native_cache_directory;
use crate::video::native_models::NativeVideoDownload;
use crate::video::progressive_route::ProgressiveState;
use log::warn;
use nostr_sdk::{Client, Filter, Kind};
use std::{future::Future, io, path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, sync::watch};

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
        // Never `Client::default()`: its database stores no events, so
        // every query would be a cold network round while ndk answers
        // from cache UNION network (crate::discovery::event_cache).
        let client = Arc::new(client_with_event_cache());
        let videos = NativeVideoIndex::new(MAX_NATIVE_INVENTORY_ITEMS);
        spawn_event_identity_indexer(client.clone(), videos.clone());
        configure_client(&client, &configuration.relays).await;
        let hls_sessions = HlsSessions::production();
        let (router, delivery, progressive, modes) =
            start_progressive_delivery(&configuration, hls_sessions.clone()).await?;
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
