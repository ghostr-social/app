//! Process-wide home of the started engine. Both `ffi_start_engine`
//! and the deprecated `ffi_start_server` alias install here, so every
//! FFI function sees the same runtime whichever start path ran.

use crate::api::tracked_items::TrackedItems;
use crate::video::gateway_runtime::{GatewayConfiguration, GatewayRuntime};
use anyhow::bail;
use flutter_rust_bridge::frb;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Everything the FFI layer holds onto after a successful start.
#[frb(ignore)]
pub(crate) struct EngineHandles {
    /// Loopback `host:port` of the embedded gateway.
    pub endpoint: String,
    pub gateway: Arc<GatewayRuntime>,
    pub tracked: TrackedItems,
}

static INSTALLED: OnceCell<Arc<EngineHandles>> = OnceCell::new();
static STARTING: AtomicBool = AtomicBool::new(false);

/// Starts the gateway runtime once per process and installs it. A
/// second call — from either start path — is rejected.
pub(crate) async fn start_and_install(
    configuration: GatewayConfiguration,
) -> anyhow::Result<String> {
    if INSTALLED.get().is_some() || STARTING.swap(true, Ordering::AcqRel) {
        bail!("The embedded gateway is already running.");
    }
    let result = GatewayRuntime::start(configuration).await;
    STARTING.store(false, Ordering::Release);
    let (endpoint, runtime) = result?;
    install(endpoint.clone(), runtime);
    Ok(endpoint)
}

fn install(endpoint: String, runtime: GatewayRuntime) {
    let handles = EngineHandles {
        endpoint,
        gateway: Arc::new(runtime),
        tracked: TrackedItems::new(),
    };
    INSTALLED.get_or_init(|| Arc::new(handles));
}

pub(crate) fn engine() -> anyhow::Result<Arc<EngineHandles>> {
    engine_if_running().ok_or_else(|| anyhow::anyhow!("The embedded gateway is not initialized."))
}

pub(crate) fn engine_if_running() -> Option<Arc<EngineHandles>> {
    INSTALLED.get().cloned()
}
