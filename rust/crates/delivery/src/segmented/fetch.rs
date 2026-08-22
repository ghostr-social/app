use anyhow::{bail, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
#[cfg(test)]
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::header::CONTENT_TYPE;
use std::sync::Arc;
use tokio::time::Instant;
use url::Url;

mod body;
mod deadline;
mod open;
mod staged;
mod telemetry;
use body::fetch_before_total;
#[cfg(test)]
use open::open;
pub(super) use staged::{fetch_stage, StagedFetch};
pub(super) use telemetry::{FetchFailure, OriginTelemetry, SegmentedTraffic};
use telemetry::{FetchProblem, FetchProgress};
#[cfg(test)]
mod tests;

pub(super) struct FetchedObject {
    pub request_url: String,
    pub final_url: Url,
    pub body: Arc<[u8]>,
    pub content_type: Option<String>,
    pub telemetry: OriginTelemetry,
}

#[derive(Clone, Copy)]
pub(super) struct FetchSpec<'a> {
    url: &'a str,
    limit: usize,
    require_manifest: bool,
    timeouts: HlsTransferTimeouts,
    priority: PreemptionAuthority,
    admission_fence: Option<Instant>,
}

struct FetchInput<'a> {
    spec: FetchSpec<'a>,
    traffic: Option<SegmentedTraffic>,
}

#[derive(Clone, Copy)]
pub(super) struct FetchRuntime<'a> {
    requests: &'a MediaRequestExecutor,
    deadline: Instant,
    network: &'a crate::delivery_events::DeliveryNetworkStatusReader,
    progress: &'a FetchProgress,
}

impl<'a> FetchRuntime<'a> {
    pub(super) const fn new(
        requests: &'a MediaRequestExecutor,
        deadline: Instant,
        network: &'a crate::delivery_events::DeliveryNetworkStatusReader,
        progress: &'a FetchProgress,
    ) -> Self {
        Self {
            requests,
            deadline,
            network,
            progress,
        }
    }
}

#[cfg(test)]
async fn asset(
    requests: &MediaRequestExecutor,
    url: &Url,
    priority: PreemptionAuthority,
) -> std::result::Result<FetchedObject, FetchFailure> {
    asset_with_timeouts(
        requests,
        url.as_str(),
        HlsTransferTimeouts::default(),
        priority,
    )
    .await
}

#[cfg(test)]
async fn asset_with_timeouts(
    requests: &MediaRequestExecutor,
    url: &str,
    timeouts: HlsTransferTimeouts,
    priority: PreemptionAuthority,
) -> std::result::Result<FetchedObject, FetchFailure> {
    let network = crate::delivery_events::DeliveryNetworkStatusReader::new(
        crate::delivery_events::DeliveryNetworkStatus::unavailable(),
    );
    fetch(
        requests,
        FetchInput {
            spec: FetchSpec {
                url,
                limit: MAX_HLS_ASSET_BYTES,
                require_manifest: false,
                timeouts,
                priority,
                admission_fence: None,
            },
            traffic: None,
        },
        &network,
        None,
    )
    .await
}

async fn fetch(
    requests: &MediaRequestExecutor,
    input: FetchInput<'_>,
    network: &crate::delivery_events::DeliveryNetworkStatusReader,
    mut cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
) -> std::result::Result<FetchedObject, FetchFailure> {
    let spec = input.spec;
    let deadline = Instant::now() + spec.timeouts.total;
    let progress = FetchProgress::new(input.traffic);
    let runtime = FetchRuntime::new(requests, deadline, network, &progress);
    let future = fetch_before_total(runtime, spec);
    let transfer = tokio::time::timeout_at(deadline, future);
    tokio::pin!(transfer);
    let result = match cancellation.as_mut() {
        Some(cancel) => tokio::select! {
            biased;
            _ = cancel => None,
            result = &mut transfer => Some(result),
        },
        None => Some(transfer.await),
    };
    let Some(result) = result else {
        return Err(FetchFailure::cancelled(
            progress.origin(),
            progress.network_bytes(),
        ));
    };
    match result {
        Ok(Ok(object)) => Ok(object),
        Ok(Err(problem)) => Err(FetchFailure::new(problem, &progress)),
        Err(error) => Err(FetchFailure::new(
            FetchProblem::new(
                anyhow::Error::new(error).context("HLS object transfer timed out"),
                ghostr_engine::origin_model::ErrorReason::Timeout,
            ),
            &progress,
        )),
    }
}

fn content_type(response: &MediaResponse) -> Option<String> {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn require_manifest_type(response: &MediaResponse) -> Result<()> {
    let media_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if matches!(
        media_type,
        Some(
            "application/vnd.apple.mpegurl"
                | "application/x-mpegurl"
                | "audio/mpegurl"
                | "audio/x-mpegurl"
        )
    ) {
        return Ok(());
    }
    bail!("HLS manifest has an unsupported content type")
}
