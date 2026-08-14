//! The manager's spawned IO: chunk downloads and HEAD probes report
//! back over the internal event channel. Each task records into a
//! scratch `HostStats`; the manager re-records outcomes into the one
//! owned instance, keeping the statistics single-owner and lock-free.

use crate::chunk::cancel::{cancel_pair, CancelHandle};
use crate::chunk::downloader::{download_chunk_observed, ChunkResult, ChunkSpec};
use crate::chunk::sink::TransferChunkSink;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::retry::CooldownId;
use crate::manager::traffic::{TrafficPublisher, TransferKey};
use crate::probe::media::{probe, ProbeResult};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

pub(crate) enum InternalEvent {
    Transfer(TransferEvent),
    Maintenance(MaintenanceEvent),
    TrafficChanged,
}

pub(crate) enum TransferEvent {
    ChunkDone(ChunkDone),
    BodyFinished(TransferIdentity),
    ProbeDone(ProbeDone),
}

pub(crate) enum MaintenanceEvent {
    CooldownOver(PostId, CooldownId),
    SaveStats,
    StoreCapacityChanged(u64),
}

pub(crate) struct ChunkDone {
    pub attempt: ChunkAttempt,
    pub url: String,
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
    pub client: Arc<dyn MediaHttpRequests>,
    pub store: Arc<PartialRangeStore>,
    pub events: UnboundedSender<InternalEvent>,
    pub timeouts: TransferTimeouts,
    pub network: NetworkThrottle,
    pub traffic: TrafficPublisher,
}

/// Starts one granted chunk transfer; the returned handle cancels it.
pub(crate) fn spawn_chunk(
    ctx: TransferContext,
    attempt: ChunkAttempt,
    url: String,
) -> CancelHandle {
    let (handle, token) = cancel_pair();
    tokio::spawn(async move {
        let spec = ChunkSpec {
            client: ctx.client.as_ref(),
            url: &url,
            range: attempt.chunk.range,
            timeouts: ctx.timeouts,
        };
        let sink = TransferChunkSink::new(&ctx.store, attempt.identity().clone());
        let mut scratch = HostStats::new();
        let mut traffic = TransferTraffic::new(attempt.id(), &url, ctx.traffic.clone());
        let outcome = download_chunk_observed(
            &spec,
            &sink,
            &mut scratch,
            &token,
            &ctx.network,
            &mut traffic,
        )
        .await;
        attempt.mark_io_finished();
        drop(traffic);
        let event = chunk_event(attempt, url, outcome);
        let _ = ctx.events.send(InternalEvent::Transfer(event));
    });
    handle
}

struct TransferTraffic {
    transfer: TransferKey,
    host: Option<String>,
    publisher: TrafficPublisher,
    opened: bool,
}

impl TransferTraffic {
    fn new(id: u64, url: &str, publisher: TrafficPublisher) -> Self {
        Self {
            transfer: TransferKey::new(id),
            host: ghostr_engine::host_stats::host_of(url),
            publisher,
            opened: false,
        }
    }
}

impl ChunkTraffic for TransferTraffic {
    fn opened(&mut self, ttfb: Duration) {
        let Some(host) = self.host.take() else {
            return;
        };
        self.opened = self
            .publisher
            .opened(self.transfer, host, ttfb, Instant::now());
    }

    fn wrote(&mut self, bytes: u64) {
        if self.opened {
            self.publisher
                .progress(self.transfer, bytes, Instant::now());
        }
    }
}

impl Drop for TransferTraffic {
    fn drop(&mut self) {
        if self.opened {
            self.publisher.closed(self.transfer, Instant::now());
        }
    }
}

pub(crate) fn chunk_event(
    attempt: ChunkAttempt,
    url: String,
    outcome: anyhow::Result<ChunkResult>,
) -> TransferEvent {
    if cancelled_before_request(&outcome) {
        return TransferEvent::BodyFinished(attempt.identity().clone());
    }
    TransferEvent::ChunkDone(ChunkDone {
        attempt,
        url,
        outcome,
    })
}

fn cancelled_before_request(outcome: &anyhow::Result<ChunkResult>) -> bool {
    outcome.as_ref().is_ok_and(|result| !result.request_started)
}

/// Starts one HEAD probe for a post whose size is still unknown.
pub(crate) fn spawn_probe(ctx: TransferContext, post: PostId, url: String) {
    tokio::spawn(async move {
        let mut scratch = HostStats::new();
        let outcome = probe(ctx.client.as_ref(), &url, ctx.timeouts, &mut scratch).await;
        let _ = ctx
            .events
            .send(InternalEvent::Transfer(TransferEvent::ProbeDone(
                ProbeDone { post, url, outcome },
            )));
    });
}
