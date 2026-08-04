//! Engine lifecycle and live configuration (plan §2 rows 1–2).

use crate::api::focus_mapping::parse_data_usage;
use crate::api::runtime_registry;
use crate::engine::{DataUsageLevel, EngineParams};
use crate::video::gateway_runtime::GatewayConfiguration;
use flutter_rust_bridge::frb;
use std::path::PathBuf;

/// Starts the media engine and returns the loopback endpoint as
/// `host:port`. Replaces `ffi_start_server`; download concurrency is
/// owned by the engine's parameter table, scaled by `data_usage`
/// (`"conservative"` / `"balanced"` / `"aggressive"`).
#[frb]
pub async fn ffi_start_engine(
    cache_directory: String,
    relay_urls: String,
    data_usage: String,
    max_storage_bytes: u64,
) -> anyhow::Result<String> {
    let level = parse_data_usage(&data_usage)?;
    let configuration = engine_configuration(cache_directory, &relay_urls, max_storage_bytes);
    let endpoint = runtime_registry::start_and_install(configuration).await?;
    apply_level(level)?;
    Ok(endpoint)
}

/// Live data-usage change. `max_storage_bytes` is validated only in
/// this slice: progressive-store eviction is not enforced yet (plan
/// divergence), so the budget cannot resize a running store.
#[frb]
pub async fn ffi_set_delivery_config(
    data_usage: String,
    max_storage_bytes: u64,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        max_storage_bytes > 0,
        "the storage budget must be positive"
    );
    let level = parse_data_usage(&data_usage)?;
    apply_level(level)
}

fn engine_configuration(
    cache_directory: String,
    relay_urls: &str,
    max_storage_bytes: u64,
) -> GatewayConfiguration {
    GatewayConfiguration {
        cache_directory: PathBuf::from(cache_directory),
        relays: relay_list(relay_urls),
        max_parallel_downloads: EngineParams::default().balanced_concurrency,
        max_storage_bytes,
    }
}

fn apply_level(level: DataUsageLevel) -> anyhow::Result<()> {
    let engine = runtime_registry::engine()?;
    engine.tracked.set_level(level);
    engine.gateway.delivery().set_data_usage(level);
    Ok(())
}

/// One relay URL per line; blank lines are ignored.
pub(crate) fn relay_list(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|relay| !relay.is_empty())
        .map(str::to_owned)
        .collect()
}
