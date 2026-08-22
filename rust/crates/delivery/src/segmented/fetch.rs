use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::strong_etag::StrongEtag;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::header::CONTENT_TYPE;
use std::sync::Arc;
use tokio::time::Instant;
use url::Url;

mod body;
mod deadline;
mod failure_policy;
mod open;
mod response;
mod staged;
mod telemetry;
use body::fetch_before_total;
pub(crate) use failure_policy::FailureDisposition;
#[cfg(test)]
use open::open;
#[cfg(test)]
pub(super) use staged::fetch_stage;
pub(super) use staged::{fetch_stage_tracked, StagedFetch};
use telemetry::FetchProblem;
pub(super) use telemetry::{FetchFailure, FetchProgress, OriginTelemetry, SegmentedTraffic};
#[cfg(test)]
mod tests;

pub(super) struct FetchedObject {
    pub request_url: String,
    pub final_url: Url,
    pub body: Arc<[u8]>,
    pub content_type: Option<String>,
    pub cache: super::cache::HlsCacheMetadata,
    pub telemetry: OriginTelemetry,
    pub offset: u64,
    pub continuation: Option<ObjectContinuation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ObjectContinuation {
    pub next_offset: u64,
    pub total: u64,
    pub final_url: Url,
    pub strong_etag: StrongEtag,
}

#[derive(Clone, Copy, Default)]
struct ObjectRequest<'a> {
    offset: u64,
    total: Option<u64>,
    final_url: Option<&'a Url>,
    strong_etag: Option<&'a StrongEtag>,
}

#[derive(Clone, Copy)]
pub(super) struct FetchSpec<'a> {
    url: &'a str,
    limit: usize,
    object_limit: u64,
    object: ObjectRequest<'a>,
    timeouts: HlsTransferTimeouts,
    priority: PreemptionAuthority,
    admission_fence: Option<Instant>,
}

impl FetchSpec<'_> {
    fn request_end(self) -> u64 {
        self.object
            .offset
            .saturating_add(self.limit as u64)
            .min(self.object_limit)
            .min(self.object.total.unwrap_or(u64::MAX))
    }
}

#[cfg(test)]
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
async fn fetch(
    requests: &MediaRequestExecutor,
    input: FetchInput<'_>,
    network: &crate::delivery_events::DeliveryNetworkStatusReader,
    cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
) -> std::result::Result<FetchedObject, FetchFailure> {
    let spec = input.spec;
    let deadline = Instant::now() + spec.timeouts.total;
    let progress = FetchProgress::new(input.traffic);
    let runtime = FetchRuntime::new(requests, deadline, network, &progress);
    fetch_tracked(runtime, spec, cancellation).await
}

pub(super) async fn fetch_tracked(
    runtime: FetchRuntime<'_>,
    spec: FetchSpec<'_>,
    cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
) -> std::result::Result<FetchedObject, FetchFailure> {
    let result = await_transfer(runtime, spec, cancellation).await;
    let Some(result) = result else {
        return Err(FetchFailure::cancelled(
            runtime.progress.origin(),
            runtime.progress.network_bytes(),
        ));
    };
    finish_transfer(result, runtime.progress)
}

type TimedFetch = std::result::Result<
    std::result::Result<FetchedObject, FetchProblem>,
    tokio::time::error::Elapsed,
>;

async fn await_transfer(
    runtime: FetchRuntime<'_>,
    spec: FetchSpec<'_>,
    mut cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
) -> Option<TimedFetch> {
    let future = fetch_before_total(runtime, spec);
    let transfer = tokio::time::timeout_at(runtime.deadline, future);
    tokio::pin!(transfer);
    match cancellation.as_mut() {
        Some(cancel) => tokio::select! {
            biased;
            _ = cancel => None,
            result = &mut transfer => Some(result),
        },
        None => Some(transfer.await),
    }
}

fn finish_transfer(
    result: TimedFetch,
    progress: &FetchProgress,
) -> std::result::Result<FetchedObject, FetchFailure> {
    match result {
        Ok(Ok(object)) => Ok(object),
        Ok(Err(problem)) => Err(FetchFailure::new(problem, progress)),
        Err(error) => Err(total_timeout(error, progress)),
    }
}

fn total_timeout(error: tokio::time::error::Elapsed, progress: &FetchProgress) -> FetchFailure {
    let error = anyhow::Error::new(error).context("HLS object transfer timed out");
    let problem = match progress.has_admission() {
        true => FetchProblem::new(error, ghostr_engine::origin_model::ErrorReason::Timeout),
        false => FetchProblem::neutral(error, ghostr_engine::origin_model::ErrorReason::Timeout),
    };
    FetchFailure::new(problem, progress)
}

fn content_type(response: &MediaResponse) -> Option<String> {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}
