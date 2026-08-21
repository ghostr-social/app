//! The manager's spawned IO: chunk downloads and HEAD probes report
//! back over the internal event channel. Each task records into a
//! scratch `HostStats`; the manager re-records outcomes into the one
//! owned instance, keeping the statistics single-owner and lock-free.

use crate::chunk::cancel::CancelToken;
use crate::chunk::downloader::{
    download_chunk_captured, ChunkResult, ChunkSpec, ObservedChunk, ResponseObservation,
};
use crate::chunk::sink::TransferChunkSink;
use crate::debug::network::NetworkThrottle;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::response_open::ResponseOpener;
use crate::manager::retry::CooldownId;
use crate::manager::traffic::TrafficPublisher;
use crate::probe::media::{probe, ProbeResult};
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::PostId;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use ghostr_partial_store::partial_range_store::StoreAction;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

mod traffic;
use traffic::TransferTraffic;

pub(crate) enum InternalEvent {
    Transfer(TransferEvent),
    Segmented(SegmentedDone),
    Maintenance(MaintenanceEvent),
    TrafficChanged,
}

pub(crate) struct SegmentedDone {
    pub post: PostId,
    pub generation: u64,
}

pub(crate) enum TransferEvent {
    ChunkDone(ChunkDone),
    ProbeDone(ProbeDone),
    ResponseObserved(ObservedResponse),
}

pub(crate) struct ObservedResponse {
    pub attempt: ChunkAttempt,
    pub response: ResponseObservation,
}

pub(crate) enum MaintenanceEvent {
    CooldownOver(PostId, CooldownId),
    SaveStats,
    SaveQoe,
    StoreCapacityChanged(u64),
}

pub(crate) struct ChunkDone {
    pub attempt: ChunkAttempt,
    pub url: String,
    pub outcome: anyhow::Result<ChunkResult>,
    pub origin: Option<Box<ghostr_engine::origin_model::OriginObservation>>,
}

pub(crate) struct ProbeDone {
    pub post: PostId,
    pub url: String,
    pub outcome: anyhow::Result<ProbeResult>,
}

/// Everything a spawned transfer needs; cheap to clone per task.
#[derive(Clone)]
pub(crate) struct TransferContext {
    pub client: Arc<dyn MediaHttpRequests>,
    pub store: Arc<PartialRangeStore>,
    pub events: UnboundedSender<InternalEvent>,
    pub responses: ResponseOpener,
    pub timeouts: TransferTimeouts,
    pub network: NetworkThrottle,
    pub traffic: TrafficPublisher,
}

pub(crate) struct ChunkLaunch {
    pub context: TransferContext,
    pub attempt: ChunkAttempt,
    pub url: String,
    pub retrieval: RetrievalRequest,
    pub token: CancelToken,
    pub action: StoreAction,
}

/// Starts one granted transfer under a supervisor that always releases it.
pub(crate) fn spawn_chunk(launch: ChunkLaunch) {
    tokio::spawn(async move {
        let attempt = launch.attempt.clone();
        let url = launch.url.clone();
        let action = launch.action.clone();
        let context = launch.context.clone();
        let worker = tokio::spawn(run_chunk(launch));
        let observed = worker.await;
        attempt.mark_io_finished();
        context.store.release_action(&action).await;
        let event = match observed {
            Ok(Ok(observed)) => observed_chunk_event(attempt, url, observed),
            Ok(Err(error)) => chunk_event(attempt, url, Err(error)),
            Err(error) => chunk_event(
                attempt,
                url,
                Err(anyhow::anyhow!("video transfer task failed: {error}")),
            ),
        };
        let _ = context.events.send(InternalEvent::Transfer(event));
    });
}

async fn run_chunk(launch: ChunkLaunch) -> anyhow::Result<ObservedChunk> {
    let ChunkLaunch {
        context,
        attempt,
        url,
        retrieval,
        token,
        action,
    } = launch;
    let sink = TransferChunkSink::new(&context.store, attempt.identity().clone(), action.clone());
    let mut scratch = HostStats::new();
    let mut traffic = TransferTraffic::new(&attempt, &context, &url, action);
    let continuation = context.store.continuation_for(attempt.identity()).await?;
    let spec = ChunkSpec {
        client: context.client.as_ref(),
        url: &url,
        request: retrieval,
        continuation: continuation.as_ref(),
        timeouts: context.timeouts,
    };
    Ok(download_chunk_captured(
        &spec,
        &sink,
        &mut scratch,
        &token,
        &context.network,
        &mut traffic,
    )
    .await)
}

pub(crate) fn chunk_event(
    attempt: ChunkAttempt,
    url: String,
    outcome: anyhow::Result<ChunkResult>,
) -> TransferEvent {
    TransferEvent::ChunkDone(ChunkDone {
        attempt,
        url,
        outcome,
        origin: None,
    })
}

fn observed_chunk_event(
    attempt: ChunkAttempt,
    url: String,
    observed: ObservedChunk,
) -> TransferEvent {
    TransferEvent::ChunkDone(ChunkDone {
        attempt,
        url,
        outcome: observed.result,
        origin: Some(Box::new(observed.origin)),
    })
}

/// Starts one HEAD probe for a post whose size is still unknown.
pub(crate) fn spawn_probe(ctx: TransferContext, post: PostId, url: String) {
    tokio::spawn(async move {
        let events = ctx.events.clone();
        let worker = tokio::spawn(run_probe(ctx, url.clone()));
        let outcome = match worker.await {
            Ok(outcome) => outcome,
            Err(error) => Err(anyhow::anyhow!("video probe task failed: {error}")),
        };
        let event = TransferEvent::ProbeDone(ProbeDone { post, url, outcome });
        let _ = events.send(InternalEvent::Transfer(event));
    });
}

async fn run_probe(ctx: TransferContext, url: String) -> anyhow::Result<ProbeResult> {
    let mut scratch = HostStats::new();
    probe(ctx.client.as_ref(), &url, ctx.timeouts, &mut scratch).await
}
