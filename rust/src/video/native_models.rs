use crate::video::ffi_models::FfiNostrEventIdentity;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct NativeUserData {
    pub npub: Option<String>,
    pub name: Option<String>,
    pub profile_picture: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NativeVideo {
    pub id: String,
    pub user: NativeUserData,
    pub title: String,
    pub song_name: String,
    pub comments: String,
    pub likes: String,
    pub url: String,
    pub delivery: NativeVideoDelivery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeVideoDelivery {
    Progressive,
    Hls,
}

impl NativeVideoDelivery {
    pub fn can_cache_as_single_file(self) -> bool {
        self == Self::Progressive
    }
}

#[derive(Clone, Debug)]
pub struct NativeVideoDownload {
    pub id: String,
    pub url: String,
    pub local_path: Option<PathBuf>,
    pub downloading: bool,
    pub event: FfiNostrEventIdentity,
    pub nostr: NativeVideo,
}

impl NativeVideoDownload {
    pub fn new(inventory_id: String, nostr: NativeVideo, event: FfiNostrEventIdentity) -> Self {
        let downloading = nostr.delivery.can_cache_as_single_file();
        Self {
            id: inventory_id,
            url: nostr.url.clone(),
            local_path: None,
            downloading,
            event,
            nostr,
        }
    }
}

pub type NativeDownloads = Arc<Mutex<HashMap<String, NativeVideoDownload>>>;

pub fn new_native_downloads() -> NativeDownloads {
    Arc::new(Mutex::new(HashMap::new()))
}
