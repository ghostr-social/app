use ghostr_engine::inventory_controller::Mode;
use ghostr_delivery::delivery_events::DeliveryHandle;
use crate::gateway_delivery::start_progressive_delivery;
use crate::hls_playback_gateway::{HlsPlaybackGateway, NativeHlsPlaybackSession};
use crate::hls_sessions::HlsSessions;
use ghostr_media_store::native_cache::prepare_native_cache_directory;
use crate::progressive_route::ProgressiveState;
use log::warn;
use nostr_sdk::Client;
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
    delivery: DeliveryHandle,
    progressive: Arc<ProgressiveState>,
}

impl GatewayRuntime {
    /// Starts everything: endpoint, runtime, and the remaining discovery
    /// boot input (the inventory-mode watch, plan §5.4). The shared Nostr
    /// client is supplied by the caller so this crate stays free of
    /// discovery.
    ///
    /// Contract on `client`: build it with `client_with_event_cache()`,
    /// never `Client::default()` — the shared client must retain verified
    /// events for session-scoped cache union and deduplication.
    pub async fn start(
        configuration: GatewayConfiguration,
        client: Arc<Client>,
    ) -> anyhow::Result<(String, Self, watch::Receiver<Mode>)> {
        validate(&configuration)?;
        prepare_native_cache_directory(&configuration.cache_directory)?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let endpoint = address.to_string();
        let hls_sessions = HlsSessions::production();
        let (router, delivery, progressive, modes) =
            start_progressive_delivery(&configuration, hls_sessions.clone(), client.clone())
                .await?;
        spawn_http_server(listener, router);
        let hls = HlsPlaybackGateway::new(address, hls_sessions);
        let runtime = Self {
            hls,
            delivery,
            progressive,
        };
        Ok((endpoint, runtime, modes))
    }

    /// Control surface for focus, demand, and data-usage updates.
    pub fn delivery(&self) -> DeliveryHandle {
        self.delivery.clone()
    }

    /// Progressive plumbing (store/demand/posts) for the FFI layer.
    pub fn progressive(&self) -> Arc<ProgressiveState> {
        self.progressive.clone()
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
