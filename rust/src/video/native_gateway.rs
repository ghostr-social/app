use crate::api::engine_control::relay_list;
use crate::api::runtime_registry;
use crate::video::ffi_models::{ffi_hls_playback_session, ffi_video_download};
use crate::video::gateway_runtime::GatewayConfiguration;
use flutter_rust_bridge::frb;
use log::warn;
use std::path::PathBuf;

pub use crate::video::ffi_models::{
    FfiHlsPlaybackSession, FfiNostrEventIdentity, FfiNostrVideo, FfiUserData, FfiVideoDelivery,
    FfiVideoDownload,
};

/// Deprecated alias for `ffi_start_engine` (plan §2): same start path
/// and shared runtime, with the data-usage level left at its Balanced
/// default. Kept only until the Dart wiring moves to the new call.
#[frb]
pub async fn ffi_start_server(
    cache_directory: String,
    max_parallel_downloads: usize,
    max_storage_bytes: u64,
    relay_urls: String,
) -> anyhow::Result<String> {
    let configuration = GatewayConfiguration {
        cache_directory: PathBuf::from(cache_directory),
        relays: relay_list(&relay_urls),
        max_parallel_downloads,
        max_storage_bytes,
    };
    runtime_registry::start_and_install(configuration).await
}

#[frb]
pub async fn ffi_get_discovered_videos() -> Vec<FfiVideoDownload> {
    let Some(engine) = runtime_registry::engine_if_running() else {
        warn!("Embedded video server is not initialized");
        return Vec::new();
    };
    engine
        .gateway
        .discovered_videos()
        .await
        .iter()
        .map(ffi_video_download)
        .collect()
}

#[frb]
pub async fn ffi_acquire_hls_playback(
    source_urls: Vec<String>,
) -> anyhow::Result<FfiHlsPlaybackSession> {
    let engine = runtime_registry::engine()?;
    let session = engine.gateway.acquire_hls(source_urls).await?;
    Ok(ffi_hls_playback_session(session))
}

#[frb]
pub async fn ffi_release_hls_playback(session_id: String) -> bool {
    match runtime_registry::engine_if_running() {
        Some(engine) => engine.gateway.release_hls(&session_id).await,
        None => false,
    }
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
