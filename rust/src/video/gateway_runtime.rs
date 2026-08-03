use crate::video::event_identity::VIDEO_KINDS;
pub use crate::video::event_index::MAX_NATIVE_INVENTORY_ITEMS;
use crate::video::event_index::{spawn_event_identity_indexer, NativeVideoIndex};
use crate::video::hls_playback_gateway::{HlsPlaybackGateway, NativeHlsPlaybackSession};
use crate::video::hls_sessions::HlsSessions;
use crate::video::http_gateway::configured_router_with_hls_sessions;
use crate::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use crate::video::native_models::{new_native_downloads, NativeDownloads, NativeVideoDownload};
use crate::video::video_manager::NativeVideoManager;
use log::warn;
use nostr_sdk::{Client, Filter, Kind};
use std::path::PathBuf;
use std::sync::Arc;
use std::{future::Future, io, time::Duration};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

const NATIVE_CACHE_EVICTION_GRACE: Duration = Duration::from_secs(30);

pub struct GatewayConfiguration {
    pub cache_directory: PathBuf,
    pub relays: Vec<String>,
    pub max_parallel_downloads: usize,
    pub max_storage_bytes: u64,
}

pub struct GatewayRuntime {
    downloads: NativeDownloads,
    hls: HlsPlaybackGateway,
    videos: NativeVideoIndex,
}

impl GatewayRuntime {
    pub async fn start(configuration: GatewayConfiguration) -> anyhow::Result<(String, Self)> {
        validate(&configuration)?;
        prepare_native_cache_directory(&configuration.cache_directory)?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let endpoint = address.to_string();
        let client = Arc::new(Client::default());
        let videos = NativeVideoIndex::new(MAX_NATIVE_INVENTORY_ITEMS);
        spawn_event_identity_indexer(client.clone(), videos.clone());
        configure_client(&client, &configuration.relays).await;
        let downloads = new_native_downloads();
        start_manager(&configuration, downloads.clone(), videos.clone())?;
        let hls_sessions = HlsSessions::production();
        let router = configured_router_with_hls_sessions(downloads.clone(), hls_sessions.clone())?;
        spawn_http_server(listener, router);
        let hls = HlsPlaybackGateway::new(address, hls_sessions);
        Ok((
            endpoint,
            Self {
                downloads,
                hls,
                videos,
            },
        ))
    }

    pub async fn discovered_videos(&self) -> Vec<NativeVideoDownload> {
        let ids = self.videos.ordered_ids().await;
        let downloads = self.downloads.lock().await;
        ids.iter()
            .filter_map(|id| downloads.get(id))
            .cloned()
            .collect()
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
    let kinds = VIDEO_KINDS
        .into_iter()
        .map(Kind::Custom)
        .collect::<Vec<_>>();
    Filter::new().kinds(kinds).limit(MAX_NATIVE_INVENTORY_ITEMS)
}

pub fn deletion_filter() -> Filter {
    Filter::new()
        .kind(Kind::EventDeletion)
        .limit(MAX_NATIVE_INVENTORY_ITEMS)
}

fn start_manager(
    configuration: &GatewayConfiguration,
    downloads: NativeDownloads,
    videos: NativeVideoIndex,
) -> anyhow::Result<()> {
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::with_eviction_grace(
        configuration.cache_directory.clone(),
        configuration.max_storage_bytes,
        used_bytes,
        NATIVE_CACHE_EVICTION_GRACE,
    );
    NativeVideoManager::new(
        downloads,
        cache,
        videos,
        configuration.max_parallel_downloads,
    )?
    .start();
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
