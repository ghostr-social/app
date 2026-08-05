#![allow(dead_code)]

pub mod content_type;
pub mod ranged;
pub mod reject;
pub mod stall;

use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub fn media_client() -> MediaHttpClient {
    MediaHttpClient::trusted().expect("trusted media client")
}

pub fn temp_root(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

pub fn store(root: PathBuf) -> PartialRangeStore {
    PartialRangeStore::new(root, Arc::new(Mutex::new(0)))
}

pub fn body() -> Vec<u8> {
    b"0123456789abcdef".to_vec()
}
