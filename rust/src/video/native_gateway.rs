use crate::video::gateway_runtime::{GatewayConfiguration, GatewayRuntime};
use anyhow::bail;
use flutter_rust_bridge::frb;
use log::warn;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use crate::video::ffi_models::{
    FfiNostrEventIdentity, FfiNostrVideo, FfiUserData, FfiVideoDelivery, FfiVideoDownload,
};

static GLOBAL_STATE: OnceCell<Arc<GatewayRuntime>> = OnceCell::new();
static STARTING: AtomicBool = AtomicBool::new(false);

#[frb]
pub async fn ffi_start_server(
    cache_directory: String,
    max_parallel_downloads: usize,
    max_storage_bytes: u64,
    relay_urls: String,
) -> anyhow::Result<String> {
    if GLOBAL_STATE.get().is_some() || STARTING.swap(true, Ordering::AcqRel) {
        bail!("The embedded gateway is already running.");
    }
    let configuration = GatewayConfiguration {
        cache_directory: PathBuf::from(cache_directory),
        relays: relay_list(&relay_urls),
        max_parallel_downloads,
        max_storage_bytes,
    };
    let result = GatewayRuntime::start(configuration).await;
    STARTING.store(false, Ordering::Release);
    let (endpoint, runtime) = result?;
    Ok(install_runtime(endpoint, runtime))
}

fn install_runtime(endpoint: String, runtime: GatewayRuntime) -> String {
    GLOBAL_STATE.get_or_init(|| Arc::new(runtime));
    endpoint
}

#[frb]
pub async fn ffi_get_discovered_videos() -> Vec<FfiVideoDownload> {
    let Some(gateway) = GLOBAL_STATE.get() else {
        warn!("Embedded video server is not initialized");
        return Vec::new();
    };
    gateway.discovered_videos().await
}

fn relay_list(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|relay| !relay.is_empty())
        .map(str::to_owned)
        .collect()
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(
                android_logger::FilterBuilder::new()
                    .parse("debug,mp4parse=off,nostr_relay_pool=off,hyper_util=off,reqwest=off")
                    .build(),
            ),
    );
}
