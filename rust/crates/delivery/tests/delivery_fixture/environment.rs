//! Test-only HTTP and filesystem environment for delivery fixtures.

use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LocalMediaClient(reqwest::Client);

impl MediaHttpRequests for LocalMediaClient {
    fn get(&self, url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub fn media_client() -> MediaRequestExecutor {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("local media client");
    MediaRequestExecutor::new(
        std::sync::Arc::new(LocalMediaClient(client)),
        MediaRequestLimits::try_new(4, 4).expect("valid test fixture"),
    )
}

pub fn temp_directory(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}
