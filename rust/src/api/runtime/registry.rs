//! Process-wide home of the started engine.

use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::discovery::cache::{client_with_event_cache, EventCache};
use anyhow::bail;
use core::sync::atomic::{AtomicBool, Ordering};
use flutter_rust_bridge::frb;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use once_cell::sync::{Lazy, OnceCell};
use std::sync::Arc;

/// Everything the FFI layer holds onto after a successful start.
#[frb(ignore)]
pub(crate) struct EngineHandles {
    /// Loopback `host:port` of the embedded gateway.
    pub endpoint: String,
    pub gateway: Arc<GatewayRuntime>,
    pub tracked: TrackedItems,
    pub discovery: DiscoveryRuntime,
}

static INSTALLED: OnceCell<Arc<EngineHandles>> = OnceCell::new();
static START_GATE: Lazy<Arc<StartGate>> = Lazy::new(|| Arc::new(StartGate::new()));

#[frb(ignore)]
pub(crate) struct StartGate {
    starting: AtomicBool,
}

#[frb(ignore)]
pub(crate) struct StartPermit {
    gate: Arc<StartGate>,
}

/// Starts the gateway runtime once per process, boots discovery on
/// its client and adaptive discovery demand, and installs both. A second call —
/// from either start path — is rejected.
pub(crate) async fn start_and_install(
    configuration: GatewayConfiguration,
    search_relays: Vec<String>,
) -> anyhow::Result<String> {
    let _permit = START_GATE.acquire(|| INSTALLED.get().is_some())?;
    let bootstrap = configuration.relays.clone();
    let cache_root = configuration.cache_directory.clone();
    // Never `Client::default()`: the shared client must retain verified
    // events for session-scoped cache union and deduplication.
    let client = Arc::new(client_with_event_cache());
    let (endpoint, runtime, demand) =
        GatewayRuntime::start(configuration, std::sync::Arc::clone(&client)).await?;
    let cache = Arc::new(EventCache::persistent(&cache_root));
    let discovery = DiscoveryRuntime::start_with_cache(
        DiscoveryBoot {
            client,
            demand,
            bootstrap,
            search_relays,
            candidates: Some(runtime.delivery()),
        },
        cache,
    )
    .await;
    install(endpoint.clone(), runtime, discovery);
    Ok(endpoint)
}

fn install(endpoint: String, runtime: GatewayRuntime, discovery: DiscoveryRuntime) {
    let handles = EngineHandles {
        endpoint,
        gateway: Arc::new(runtime),
        tracked: TrackedItems::new(),
        discovery,
    };
    INSTALLED.get_or_init(|| Arc::new(handles));
}

pub(crate) fn engine() -> anyhow::Result<Arc<EngineHandles>> {
    engine_if_running().ok_or_else(|| anyhow::anyhow!("The embedded gateway is not initialized."))
}

pub(crate) fn engine_if_running() -> Option<Arc<EngineHandles>> {
    INSTALLED.get().cloned()
}

impl StartGate {
    pub(crate) fn new() -> Self {
        Self {
            starting: AtomicBool::new(false),
        }
    }

    pub(crate) fn acquire(
        self: &Arc<Self>,
        is_installed: impl FnOnce() -> bool,
    ) -> anyhow::Result<StartPermit> {
        let acquired = self
            .starting
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        if !acquired {
            bail!("The embedded gateway is already running.");
        }
        let permit = StartPermit {
            gate: std::sync::Arc::clone(self),
        };
        if is_installed() {
            bail!("The embedded gateway is already running.");
        }
        Ok(permit)
    }
}

impl Drop for StartPermit {
    fn drop(&mut self) {
        self.gate.starting.store(false, Ordering::Release);
    }
}
