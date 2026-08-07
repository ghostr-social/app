//! Shared scaffolding for the gateway's routed tests: a temp root, a
//! trusted media client, and a real delivery manager behind them.

#![allow(dead_code)]

pub mod debug_clear;
pub mod delivery;

use ghostr_net::outbound_media_client::MediaHttpClient;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn temp_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

pub fn media_client() -> MediaHttpClient {
    MediaHttpClient::trusted().expect("trusted media client")
}
