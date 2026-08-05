//! The manager's spawned IO: chunk downloads and HEAD probes report
//! back over the internal event channel. Each task records into a
//! scratch `HostStats`; the manager re-records outcomes into the one
//! owned instance, keeping the statistics single-owner and lock-free.

use crate::engine::host_stats::HostStats;
use crate::engine::PostId;
use crate::video::chunk_cancel::{cancel_pair, CancelHandle};
use crate::video::chunk_downloader::{download_chunk, ChunkResult, ChunkSink, ChunkSpec};
use crate::video::delivery_inflight::ChunkAttempt;
use crate::video::media_probe::{probe, ProbeResult};
use crate::video::outbound_media_client::MediaHttpClient;
use crate::video::partial_range_store::PartialRangeStore;
use crate::video::transfer_timeouts::TransferTimeouts;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

pub(crate) enum InternalEvent {
    ChunkDone(ChunkDone),
    ProbeDone(ProbeDone),
    CooldownOver(PostId),
    SaveStats,
}

pub(crate) struct ChunkDone {
    pub attempt: ChunkAttempt,
    pub url: String,
    pub elapsed: Duration,
    pub outcome: anyhow::Result<ChunkResult>,
}

pub(crate) struct ProbeDone {
    pub post: PostId,
    pub url: String,
    pub outcome: anyhow::Result<ProbeResult>,
}

/// Everything a spawned transfer needs; cheap to clone per task.
#[derive(Clone)]
pub(crate) struct TransferContext {
    pub client: MediaHttpClient,
    pub store: Arc<PartialRangeStore>,
    pub events: UnboundedSender<InternalEvent>,
    pub timeouts: TransferTimeouts,
}

/// Starts one granted chunk transfer; the returned handle cancels it.
pub(crate) fn spawn_chunk(
    ctx: TransferContext,
    attempt: ChunkAttempt,
    url: String,
) -> CancelHandle {
    let (handle, token) = cancel_pair();
    tokio::spawn(async move {
        let started = Instant::now();
        let spec = ChunkSpec {
            client: &ctx.client,
            url: &url,
            range: attempt.chunk.range,
            timeouts: ctx.timeouts,
        };
        let sink = ChunkSink {
            store: &ctx.store,
            key: attempt.chunk.post.as_str(),
        };
        let mut scratch = HostStats::new();
        let outcome = download_chunk(&spec, &sink, &mut scratch, &token).await;
        let done = ChunkDone {
            attempt,
            url,
            elapsed: started.elapsed(),
            outcome,
        };
        let _ = ctx.events.send(InternalEvent::ChunkDone(done));
    });
    handle
}

/// Starts one HEAD probe for a post whose size is still unknown.
pub(crate) fn spawn_probe(ctx: TransferContext, post: PostId, url: String) {
    tokio::spawn(async move {
        let mut scratch = HostStats::new();
        let outcome = probe(&ctx.client, &url, ctx.timeouts, &mut scratch).await;
        let _ = ctx
            .events
            .send(InternalEvent::ProbeDone(ProbeDone { post, url, outcome }));
    });
}
