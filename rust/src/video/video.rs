use std::sync::Arc;
use once_cell::sync::OnceCell;

use flutter_rust_bridge::frb; // Or use #[flutter_rust_bridge::frb], whichever you prefer
use log::warn;
use ghostr_rs::service::main_axum::start_axum_server;
use ghostr_rs::models::models::VideoDownload;
use ghostr_rs::service::state::AppState;
use android_logger;

static GLOBAL_STATE: OnceCell<Arc<AppState>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct FfiVideoDownload {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub local_path: Option<String>,
}

/// Start the Axum server and store the AppState in GLOBAL_STATE.
/// Return the bound address as a String.
#[frb]
pub async fn ffi_start_server(
    max_parallel_downloads: usize,
    max_storage_bytes: u64,
    address: Option<String>,
) -> String {
    match start_axum_server(max_parallel_downloads, max_storage_bytes, address).await {
        Ok((addr, state)) => {
            // Store the newly created AppState
            GLOBAL_STATE.set(state).ok();
            addr
        }
        Err(e) => panic!("Error starting server: {e}"),
    }
}

/// Return the discovered videos from the stored AppState.
#[frb]
pub async fn ffi_get_discovered_videos() -> Vec<FfiVideoDownload> {
    // Get the Arc<AppState> from the static
    let app_state = GLOBAL_STATE
        .get()
        .expect("Axum server not started or state not set");

    // Lock the discovered_videos
    let discovered = app_state.playlist.lock().await.new_content();
    warn!("Discovered videos: {:?}", discovered);

    discovered
        .iter()
        .map(|vid| {
            let is_fully_downloaded = match vid.content_length {
                Some(total) if total > 0 => vid.downloaded_bytes >= total,
                _ => false,
            };

            let local_path = if is_fully_downloaded {
                vid.local_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };

            FfiVideoDownload {
                id: vid.id.to_string(),
                url: vid.url.clone(),
                title: Some(vid.nostr.title.clone()),
                local_path,
            }
        })
        .collect()
}


/// Initialize your app (logging, etc.). By default, flutter_rust_bridge calls this once at startup.
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
