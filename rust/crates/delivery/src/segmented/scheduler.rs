use super::fetch::FetchFailure;
use super::SegmentedCache;
use crate::manager::traffic::TrafficPublisher;
use crate::manager::transfers::InternalEvent;
use ghostr_engine::adaptive::{HlsBootstrapStage, HlsObjectCursor};
use ghostr_engine::origin_model::OriginObservation;
use ghostr_engine::{ActionId, DeliveryKind, PostId};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

#[cfg(test)]
#[path = "scheduler/test_registry.rs"]
mod test_registry;

mod completion;
mod focus;
use focus::hls_items;
mod focus_changes;
mod launch;
mod prepared;
mod progress;
mod recovery;
mod resources;
mod root_selection;
mod snapshot;
mod target;

pub(crate) use crate::segmented::fetch::FailureDisposition;
use progress::Pending;
pub(crate) use recovery::{RecoveryAction, SegmentedRecovery, SegmentedRetry};
pub(crate) use resources::SegmentedResourceCommitment;
use target::{targets, Target};

const MAX_HLS_READY_WINDOW: usize = 5;

pub(crate) struct SegmentedDelivery {
    cache: SegmentedCache,
    tracked: Vec<(PostId, Vec<String>)>,
    targets: Vec<Target>,
    pending: BTreeMap<PostId, Pending>,
    active: BTreeMap<PostId, Active>,
    next_generation: u64,
    next_attempt: u64,
    current_delivery: Option<DeliveryKind>,
    startup_eta_ms: u64,
}

struct Active {
    action: ActionId,
    fence: crate::segmented::cache::StageFence,
    pending: Pending,
    committed_until_ms: u64,
    network: Arc<crate::segmented::fetch::FetchProgress>,
    _task: tokio::task::JoinHandle<()>,
    cancellation: Option<tokio::sync::oneshot::Sender<()>>,
    cancelling: bool,
}

pub(crate) struct SegmentedLaunch {
    pub post: PostId,
    pub stage: HlsBootstrapStage,
    pub source: String,
    pub cursor: HlsObjectCursor,
    pub maximum_bytes: u64,
    pub committed_until_ms: u64,
    pub action: ActionId,
    pub requests: MediaRequestExecutor,
    pub events: UnboundedSender<InternalEvent>,
    pub network_status: crate::delivery_events::DeliveryNetworkStatusReader,
    pub traffic: TrafficPublisher,
    pub resources: SegmentedResourceCommitment,
}

pub(crate) struct SegmentedDone {
    action: ActionId,
    post: PostId,
    fence: crate::segmented::cache::StageFence,
    outcome: Result<prepared::PreparedTransfer, FetchFailure>,
    observed_at_ms: u64,
    resources: SegmentedResourceCommitment,
}

pub(crate) struct SegmentedFinish {
    pub action: ActionId,
    pub outcome: ghostr_engine::adaptive::DecisionOutcome,
    pub observation: Option<OriginObservation>,
    pub actual_resources: Option<ghostr_engine::adaptive::ResourceCost>,
    pub resources: SegmentedResourceCommitment,
    pub recovery: SegmentedRecovery,
}

impl SegmentedDelivery {
    pub fn new(cache: SegmentedCache) -> Self {
        Self {
            cache,
            tracked: Vec::new(),
            targets: Vec::new(),
            pending: BTreeMap::new(),
            active: BTreeMap::new(),
            next_generation: 0,
            next_attempt: 0,
            current_delivery: None,
            startup_eta_ms: crate::qoe::QoeTracker::DEFAULT_STARTUP_ETA_MS,
        }
    }

    fn allocate_attempt(&mut self) -> u64 {
        self.next_attempt = self.next_attempt.wrapping_add(1).max(1);
        self.next_attempt
    }

    pub fn active_len(&self) -> usize {
        self.active
            .values()
            .filter(|active| active.network.network_active())
            .count()
    }

    pub fn set_startup_eta_ms(&mut self, eta_ms: u64) {
        self.startup_eta_ms = eta_ms;
    }

    pub fn clear(&mut self) {
        self.cancel_all();
        self.tracked.clear();
        self.targets.clear();
        self.pending.clear();
        self.current_delivery = None;
        self.cache.clear();
    }

    fn cancel_all(&mut self) {
        for active in self.active.values_mut() {
            if active.cancelling {
                continue;
            }
            if let Some(cancellation) = active.cancellation.take() {
                active.cancelling = cancellation.send(()).is_ok();
            }
        }
    }
}
