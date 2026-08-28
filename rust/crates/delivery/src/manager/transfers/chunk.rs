use super::{ChunkDone, TransferContext, TransferEvent};
use crate::chunk::cancel::CancelToken;
use crate::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSpec, ObservedChunk,
};
use crate::chunk::sink::TransferChunkSink;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::transfers::traffic::TransferTraffic;
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::SourceGeneration;
use ghostr_partial_store::partial_range_store::StoreAction;

pub(crate) struct ChunkLaunch {
    pub context: TransferContext,
    pub attempt: ChunkAttempt,
    pub url: String,
    pub retrieval: RetrievalRequest,
    pub priority: PreemptionAuthority,
    pub token: CancelToken,
    pub action: StoreAction,
    pub network_class: ghostr_engine::origin_model::NetworkClass,
}

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
        let event = supervised_event(attempt, url, observed);
        let _ = context.events.send(super::InternalEvent::Transfer(event));
    });
}

fn supervised_event(
    attempt: ChunkAttempt,
    url: String,
    observed: Result<anyhow::Result<ObservedChunk>, tokio::task::JoinError>,
) -> TransferEvent {
    match observed {
        Ok(Ok(observed)) => observed_chunk_event(attempt, url, observed),
        Ok(Err(error)) => chunk_event(attempt, url, Err(error)),
        Err(error) => chunk_event(
            attempt,
            url,
            Err(anyhow::anyhow!("video transfer task failed: {error}")),
        ),
    }
}

async fn run_chunk(launch: ChunkLaunch) -> anyhow::Result<ObservedChunk> {
    let continuation = launch
        .context
        .store
        .continuation_for(launch.attempt.identity())
        .await?;
    Ok(execute_chunk(&launch, continuation.as_ref()).await)
}

async fn execute_chunk(
    launch: &ChunkLaunch,
    continuation: Option<&SourceGeneration>,
) -> ObservedChunk {
    let sink = TransferChunkSink::new(
        &launch.context.store,
        launch.attempt.identity().clone(),
        launch.action.clone(),
    );
    let mut scratch = HostStats::new();
    let mut traffic = TransferTraffic::new(
        &launch.attempt,
        &launch.context,
        &launch.url,
        launch.action.clone(),
    );
    let spec = chunk_spec(launch, continuation);
    download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut scratch,
            cancel: &launch.token,
            network: &launch.context.network,
            traffic: &mut traffic,
            network_class: launch.network_class,
        },
    )
    .await
}

fn chunk_spec<'a>(
    launch: &'a ChunkLaunch,
    continuation: Option<&'a SourceGeneration>,
) -> ChunkSpec<'a> {
    ChunkSpec {
        requests: &launch.context.requests,
        url: &launch.url,
        request: launch.retrieval,
        attempt_profile: launch.attempt.profile(),
        priority: launch.priority,
        continuation,
        timeouts: launch.context.timeouts,
    }
}

pub(crate) fn chunk_event(
    attempt: ChunkAttempt,
    url: String,
    outcome: anyhow::Result<ChunkResult>,
) -> TransferEvent {
    let received_bytes = outcome.as_ref().map_or(0, |result| result.bytes_written);
    TransferEvent::ChunkDone(Box::new(ChunkDone {
        attempt,
        url,
        outcome,
        received_bytes,
        origin: None,
        open_body: None,
        request_started: false,
        whole_body_completion: None,
        response_evidence: None,
    }))
}

fn observed_chunk_event(
    attempt: ChunkAttempt,
    url: String,
    observed: ObservedChunk,
) -> TransferEvent {
    TransferEvent::ChunkDone(Box::new(ChunkDone {
        attempt,
        url,
        outcome: observed.result,
        received_bytes: observed.received_bytes,
        origin: observed.origin.map(Box::new),
        open_body: observed.open_body.map(Box::new),
        request_started: observed.request_started,
        whole_body_completion: observed.whole_body_completion,
        response_evidence: observed.response_evidence,
    }))
}
