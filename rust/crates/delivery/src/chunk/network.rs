//! Cancellation-aware admission into the debug network simulator.

use crate::chunk::cancel::CancelToken;
use crate::debug::network::{ConnectionPermit, NetworkThrottle};

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
    if cancel.is_cancelled() {
        return NetworkPreparation::Cancelled;
    }
    let permit = throttle.acquire(url);
    if wait_for_latency(throttle, cancel).await {
        return NetworkPreparation::Cancelled;
    }
    NetworkPreparation::Ready(Some(permit))
}

async fn wait_for_latency(network: &NetworkThrottle, cancel: &CancelToken) -> bool {
    tokio::select! {
        biased;
        () = cancel.cancelled() => true,
        () = network.wait_for_latency() => false,
    }
}
