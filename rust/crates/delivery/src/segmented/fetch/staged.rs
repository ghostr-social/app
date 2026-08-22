use super::{fetch, FetchFailure, FetchInput, FetchSpec, FetchedObject, SegmentedTraffic};
use crate::delivery_events::DeliveryNetworkStatusReader;
use ghostr_engine::adaptive::{HlsBootstrapStage, PreemptionAuthority};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use std::time::Duration;
use tokio::time::Instant;

pub(in crate::segmented) struct StagedFetch<'a> {
    pub requests: &'a MediaRequestExecutor,
    pub stage: HlsBootstrapStage,
    pub url: &'a str,
    pub priority: PreemptionAuthority,
    pub committed_until_ms: u64,
    pub network_status: &'a DeliveryNetworkStatusReader,
    pub cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
    pub traffic: Option<SegmentedTraffic>,
}

pub(in crate::segmented) async fn fetch_stage(
    input: StagedFetch<'_>,
) -> std::result::Result<FetchedObject, FetchFailure> {
    let remaining = input
        .committed_until_ms
        .saturating_sub(crate::manager::time::unix_time_ms());
    if remaining == 0 {
        return Err(FetchFailure::preflight(
            anyhow::anyhow!("HLS WARP commitment expired before launch"),
            ghostr_engine::origin_model::ErrorReason::Policy,
        ));
    }
    fetch(
        input.requests,
        FetchInput {
            spec: FetchSpec {
                url: input.url,
                limit: input.stage.maximum_bytes() as usize,
                require_manifest: input.stage.is_manifest(),
                timeouts: HlsTransferTimeouts::default(),
                priority: input.priority,
                admission_fence: Some(Instant::now() + Duration::from_millis(remaining)),
            },
            traffic: input.traffic,
        },
        input.network_status,
        input.cancellation,
    )
    .await
}
