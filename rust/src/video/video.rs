use std::sync::Arc;
use once_cell::sync::OnceCell;

use flutter_rust_bridge::frb;
use log::{info, warn};
use ghostr_rs::service::main_axum::start_axum_server;
use ghostr_rs::service::state::AppState;
use android_logger;

static GLOBAL_STATE: OnceCell<Arc<AppState>> = OnceCell::new();


#[derive(Debug, Clone)]
pub struct FfiUserData{
    pub npub: Option<String>,
    pub name: Option<String>,
    pub profile_picture: Option<String>,
}
#[derive(Debug, Clone)]
pub struct FfiNostrVideo {
    pub id: String,
    pub user: FfiUserData,
    pub title: String,
    pub song_name: String,
    pub likes: String,
    pub comments: String,
    pub url: String,
}
#[derive(Debug, Clone)]
pub struct FfiVideoDownload {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub local_path: Option<String>,
    pub nostr: FfiNostrVideo
}

/// Start the Axum server and store the AppState in GLOBAL_STATE.
/// Return the bound address as a String.
#[frb]
pub async fn ffi_start_server(
    max_parallel_downloads: usize,
    max_storage_bytes: u64) -> String {
    match start_axum_server(max_parallel_downloads, max_storage_bytes).await {
        Ok((addr, state)) => {
            // Store the Arc<AppState> in the static if not already set
            // (Usually you'd only call this function once.)
            GLOBAL_STATE.set(state).ok();
            addr
        }
        Err(e) => format!("Error starting server: {e}"),
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
            info!("Localpth: {:?}", vid.local_path);
            let local_path = if vid.local_path.is_some() && !vid.downloading {
                Some(vid.local_path.as_ref().unwrap().to_string_lossy().to_string())
            } else {
                None
            };

            FfiVideoDownload {
                id: vid.id.to_string(),
                url: vid.url.clone(),
                title: Some(vid.nostr.title.clone()),
                local_path,
                nostr: FfiNostrVideo{
                    id: vid.nostr.id.to_string(),
                    user: FfiUserData{
                        npub: vid.nostr.user.npub.clone(),
                        name: vid.nostr.user.name.clone(),
                        profile_picture: vid.nostr.user.profile_picture.clone(),
                    },
                    title: vid.nostr.title.clone(),
                    song_name: vid.nostr.song_name.clone(),
                    likes: vid.nostr.likes.clone(),
                    comments: vid.nostr.comments.clone(),
                    url: vid.nostr.url.clone(),
                },
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
