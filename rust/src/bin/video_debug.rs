#[cfg(any(not(debug_assertions), target_os = "android", target_os = "ios"))]
compile_error!("video-debug is available only in debug builds on non-mobile hosts");

use anyhow::Context;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use rust_lib_ghostr::api::debug::nostr::{DebugNostrConfiguration, DebugNostrRuntime};
use rust_lib_ghostr::discovery::cache::client_with_event_cache;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

const DEBUG_STORAGE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const DEBUG_PARALLEL_DOWNLOADS: usize = 4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nostr = DebugNostrConfiguration::from_environment()?;
    let relay_summary = nostr.read_relays.join(", ");
    let configuration = GatewayConfiguration {
        cache_directory: cache_directory(),
        relays: nostr.read_relays.clone(),
        max_parallel_downloads: DEBUG_PARALLEL_DOWNLOADS,
        max_storage_bytes: DEBUG_STORAGE_BYTES,
    };
    let client = Arc::new(client_with_event_cache());
    let (endpoint, runtime, modes) = GatewayRuntime::start(configuration, client.clone()).await?;
    let feed = runtime.progressive().debug_feed.clone();
    let _nostr = DebugNostrRuntime::start(client, modes, nostr, feed).await?;
    println!("Video debug dashboard: http://{endpoint}/debug");
    println!("Nostr discovery relays: {relay_summary}");
    io::stdout().flush().context("flush dashboard URL")?;
    tokio::signal::ctrl_c()
        .await
        .context("wait for shutdown signal")?;
    std::process::exit(0)
}

fn cache_directory() -> PathBuf {
    std::env::var_os("GHOSTR_VIDEO_DEBUG_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/video-debug-cache"))
}
