//! The manager's spawned IO: chunk downloads and HEAD probes report
//! back over the internal event channel. Each task records into a
//! scratch `HostStats`; the manager re-records outcomes into the one
//! owned instance, keeping the statistics single-owner and lock-free.

use crate::chunk::downloader::{ChunkResult, ResponseObservation};
use crate::debug::network::NetworkThrottle;
use crate::delivery_events::DecisionClaim;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::response_open::ResponseOpener;
use crate::manager::retry::CooldownId;
use crate::manager::traffic::TrafficPublisher;
use crate::probe::media::ProbeResult;
use ghostr_engine::PostId;
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

mod chunk;
mod probe;
mod traffic;
#[cfg(test)]
pub(crate) use chunk::chunk_event;
pub(crate) use chunk::{spawn_chunk, ChunkLaunch};
pub(crate) use probe::{spawn_probe, ProbeLaunch};

pub(crate) enum InternalEvent {
    ImmediateReplan,
    Transfer(TransferEvent),
    Segmented(SegmentedDone),
    Transform(crate::manager::transforms::TransformDone),
    HedgeTail(crate::manager::hedge_tail::HedgeTailWake),
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
    pub request_started: bool,
}

pub(crate) struct ProbeDone {
    pub observation: ProbeObservation,
    pub decision: DecisionClaim,
}

pub(crate) struct ProbeObservation {
    pub post: PostId,
    pub url: String,
    pub outcome: anyhow::Result<ProbeResult>,
    pub concurrency: usize,
}

/// Everything a spawned transfer needs; cheap to clone per task.
#[derive(Clone)]
pub(crate) struct TransferContext {
    pub requests: MediaRequestExecutor,
    pub store: Arc<PartialRangeStore>,
    pub events: UnboundedSender<InternalEvent>,
    pub responses: ResponseOpener,
    pub timeouts: TransferTimeouts,
    pub network: NetworkThrottle,
    pub traffic: TrafficPublisher,
}
