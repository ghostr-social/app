//! Cancellation-aware admission into the debug network simulator.

use crate::video::chunk_cancel::CancelToken;
use crate::video::debug_network::{ConnectionPermit, NetworkThrottle};

pub(crate) enum NetworkPreparation {
    Ready(Option<ConnectionPermit>),
    Cancelled,
}

pub(crate) async fn prepare_network(
    network: Option<&NetworkThrottle>,
    url: &str,
    cancel: &CancelToken,
) -> NetworkPreparation {
    let Some(throttle) = network else {
        return NetworkPreparation::Ready(None);
    };
    let permit = tokio::select! {
        biased;
        _ = cancel.cancelled() => return NetworkPreparation::Cancelled,
        permit = throttle.acquire(url) => permit,
    };
    if wait_for_latency(throttle, cancel).await {
        return NetworkPreparation::Cancelled;
    }
    NetworkPreparation::Ready(Some(permit))
}

async fn wait_for_latency(network: &NetworkThrottle, cancel: &CancelToken) -> bool {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => true,
        _ = network.wait_for_latency() => false,
    }
}
