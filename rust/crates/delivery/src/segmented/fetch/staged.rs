use super::{
    fetch_tracked, FetchFailure, FetchProgress, FetchRuntime, FetchSpec, FetchedObject,
    ObjectContinuation, ObjectRequest,
};
use crate::delivery_events::DeliveryNetworkStatusReader;
use core::time::Duration;
use ghostr_engine::adaptive::{HlsBootstrapStage, PreemptionAuthority};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use tokio::time::Instant;

pub(in crate::segmented) struct StagedFetch<'a> {
    pub requests: &'a MediaRequestExecutor,
    pub stage: HlsBootstrapStage,
    pub url: &'a str,
    pub maximum_bytes: u64,
    pub continuation: Option<&'a ObjectContinuation>,
    pub priority: PreemptionAuthority,
    pub committed_until_ms: u64,
    pub network_status: &'a DeliveryNetworkStatusReader,
    pub cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
    #[cfg(test)]
    pub(crate) traffic: Option<super::SegmentedTraffic>,
}

pub(in crate::segmented) struct TrackedStageFetch {
    pub result: core::result::Result<FetchedObject, FetchFailure>,
    pub cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
}

pub(in crate::segmented) async fn fetch_stage_tracked(
    mut input: StagedFetch<'_>,
    progress: &FetchProgress,
) -> TrackedStageFetch {
    let spec = match stage_spec(&input) {
        Ok(spec) => spec,
        Err(error) => {
            progress.finish_network();
            return TrackedStageFetch {
                result: Err(error),
                cancellation: input.cancellation,
            };
        }
    };
    let deadline = Instant::now() + spec.timeouts.total;
    let runtime = FetchRuntime::new(input.requests, deadline, input.network_status, progress);
    let result = fetch_tracked(runtime, spec, &mut input.cancellation).await;
    if result.is_ok() {
        progress.finish_response();
    } else {
        progress.finish_network();
    }
    TrackedStageFetch {
        result,
        cancellation: input.cancellation,
    }
}

fn stage_spec<'a>(input: &StagedFetch<'a>) -> core::result::Result<FetchSpec<'a>, FetchFailure> {
    if !valid_block(input) {
        return Err(preflight("invalid HLS WARP block commitment"));
    }
    if input.continuation.is_some_and(|value| {
        value.next_offset >= value.total || value.total > input.stage.maximum_bytes()
    }) {
        return Err(preflight("invalid HLS continuation cursor"));
    }
    let remaining = input
        .committed_until_ms
        .saturating_sub(crate::manager::time::unix_time_ms());
    if remaining == 0 {
        return Err(FetchFailure::preflight(
            anyhow::anyhow!("HLS WARP commitment expired before launch"),
            ghostr_engine::origin_model::ErrorReason::Policy,
        ));
    }
    Ok(FetchSpec {
        url: input.url,
        limit: input.maximum_bytes as usize,
        object_limit: input.stage.maximum_bytes(),
        object: input
            .continuation
            .map_or_else(ObjectRequest::default, |value| ObjectRequest {
                offset: value.next_offset,
                total: Some(value.total),
                final_url: Some(&value.final_url),
                strong_etag: Some(&value.strong_etag),
            }),
        timeouts: HlsTransferTimeouts::default(),
        priority: input.priority,
        admission_fence: Some(Instant::now() + Duration::from_millis(remaining)),
    })
}

fn valid_block(input: &StagedFetch<'_>) -> bool {
    let bounded = input.stage.block_bytes(input.maximum_bytes);
    let expected = input.continuation.map_or(bounded, |continuation| {
        bounded.min(continuation.total.saturating_sub(continuation.next_offset))
    });
    expected == input.maximum_bytes
}

fn preflight(message: &'static str) -> FetchFailure {
    FetchFailure::preflight(
        anyhow::anyhow!(message),
        ghostr_engine::origin_model::ErrorReason::Policy,
    )
}

#[cfg(test)]
#[path = "staged_axiom_test.rs"]
pub(crate) mod axiom_test_support;
