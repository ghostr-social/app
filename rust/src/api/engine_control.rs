//! Engine lifecycle and live configuration (plan §2 rows 1–2).

use crate::api::network_control::FfiDeliveryNetworkStatus;
use crate::api::runtime::configuration;
use crate::api::runtime::registry;
use crate::discovery::relay::url::normalize_relay_url;
use crate::engine::{DataUsageLevel, EngineParams};
use flutter_rust_bridge::frb;
use ghostr_gateway::runtime::GatewayConfiguration;
use std::collections::HashSet;
use std::path::PathBuf;

/// User-selected network and delivery pressure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiDataUsageLevel {
    Conservative,
    Balanced,
    Aggressive,
}

impl From<FfiDataUsageLevel> for DataUsageLevel {
    fn from(level: FfiDataUsageLevel) -> Self {
        match level {
            FfiDataUsageLevel::Conservative => Self::Conservative,
            FfiDataUsageLevel::Balanced => Self::Balanced,
            FfiDataUsageLevel::Aggressive => Self::Aggressive,
        }
    }
}

/// Settings shared by Nostr discovery and progressive delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfiEngineConfiguration {
    pub read_relay_urls: Vec<String>,
    pub search_relay_urls: Vec<String>,
    pub data_usage: FfiDataUsageLevel,
    pub max_storage_bytes: u64,
}

struct EngineConfiguration {
    read_relays: Vec<String>,
    search_relays: Vec<String>,
    level: DataUsageLevel,
    max_storage_bytes: u64,
}

impl TryFrom<FfiEngineConfiguration> for EngineConfiguration {
    type Error = anyhow::Error;

    fn try_from(configuration: FfiEngineConfiguration) -> anyhow::Result<Self> {
        anyhow::ensure!(
            configuration.max_storage_bytes > 0,
            "the storage budget must be positive"
        );
        Ok(Self {
            read_relays: validated_relay_urls(configuration.read_relay_urls)?,
            search_relays: validated_relay_urls(configuration.search_relay_urls)?,
            level: configuration.data_usage.into(),
            max_storage_bytes: configuration.max_storage_bytes,
        })
    }
}

/// Starts the media engine and returns the loopback endpoint as
/// `host:port`. Download concurrency is owned by the engine's parameter
/// table and scaled by `data_usage`.
#[frb]
pub async fn ffi_start_engine(
    cache_directory: String,
    configuration: FfiEngineConfiguration,
    device_integration_origin: Option<String>,
    initial_network: FfiDeliveryNetworkStatus,
) -> anyhow::Result<String> {
    let configuration = EngineConfiguration::try_from(configuration)?;
    let level = configuration.level;
    let search_relays = configuration.search_relays.clone();
    let gateway = engine_configuration(
        cache_directory,
        &configuration,
        device_integration_origin,
        initial_network,
    );
    let endpoint = registry::start_and_install(gateway, search_relays).await?;
    apply_level(level)?;
    Ok(endpoint)
}

/// Applies relay, data-usage, and progressive-storage settings without
/// restarting the engine.
#[frb]
pub async fn ffi_set_delivery_config(configuration: FfiEngineConfiguration) -> anyhow::Result<()> {
    let configuration = EngineConfiguration::try_from(configuration)?;
    let engine = registry::engine()?;
    let mut transition = engine.discovery.relay_pool.begin_configuration().await;
    engine
        .gateway
        .set_storage_budget(configuration.max_storage_bytes)
        .await?;
    configuration::replace_relays(
        &engine.discovery,
        &mut transition,
        configuration.read_relays,
        configuration.search_relays,
    )
    .await;
    apply_level(configuration.level)
}

fn engine_configuration(
    cache_directory: String,
    configuration: &EngineConfiguration,
    device_integration_origin: Option<String>,
    initial_network: FfiDeliveryNetworkStatus,
) -> GatewayConfiguration {
    GatewayConfiguration {
        cache_directory: PathBuf::from(cache_directory),
        relays: configuration.read_relays.clone(),
        max_parallel_downloads: EngineParams::default().balanced_concurrency,
        max_storage_bytes: configuration.max_storage_bytes,
        network_status: initial_network.into(),
        device_integration_origin,
    }
}

fn apply_level(level: DataUsageLevel) -> anyhow::Result<()> {
    let engine = registry::engine()?;
    engine.tracked.set_level(level);
    engine.gateway.delivery().set_data_usage(level);
    engine.discovery.set_data_usage(level);
    Ok(())
}

pub(crate) fn validated_relay_urls(raw: Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut seen = HashSet::new();
    let mut urls = Vec::new();
    for value in raw {
        let url = normalize_relay_url(&value)
            .ok_or_else(|| anyhow::anyhow!("invalid relay URL: {value}"))?;
        if seen.insert(url.clone()) {
            urls.push(url);
        }
    }
    Ok(urls)
}
