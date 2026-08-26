//! Shared fakes and builders for discovery scheduler tests: a gated
//! executor stands in for relay IO so no test ever touches the network.

use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, PlanFailure, RetrievalOutcome};
use crate::scheduler::{start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig};
use crate::session_generation::SessionGeneration;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Semaphore};

pub(super) use crate::tests::scheduler_wait::{next_outcome, next_started, no_start};

/// Executor that reports every start and holds each retrieval until a
/// gate permit is released; completed retrievals return `events`.
pub struct GatedExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    gate: Arc<Semaphore>,
    events: Vec<Event>,
}

impl PlanExecutor for GatedExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let gate = std::sync::Arc::clone(&self.gate);
        let events = self.events.clone();
        Box::pin(async move {
            let permit = gate
                .acquire()
                .await
                .map_err(|e| PlanFailure::new(e.to_string()))?;
            permit.forget();
            Ok(events)
        })
    }
}

pub(crate) struct SchedulerHarness {
    pub(super) handle: DiscoveryHandle,
    pub(super) started: mpsc::UnboundedReceiver<PlannedRetrieval>,
    pub(super) gate: Arc<Semaphore>,
    pub(super) outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
    pub(super) demand: watch::Sender<DiscoveryDemand>,
}

/// Boots a scheduler over a gated executor. Demand starts held so a
/// test-sent expansion is an observable transition.
pub(crate) fn start_scheduler(level: DataUsageLevel, events: Vec<Event>) -> SchedulerHarness {
    let (starts, started) = mpsc::unbounded_channel();
    let gate = Arc::new(Semaphore::new(0));
    let executor = Arc::new(GatedExecutor {
        starts,
        gate: std::sync::Arc::clone(&gate),
        events,
    });
    let (outcome_sender, outcomes) = mpsc::unbounded_channel();
    let (demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor,
        level,
        demand,
        outcomes: outcome_sender,
    });
    SchedulerHarness {
        handle,
        started,
        gate,
        outcomes,
        demand: demand_sender,
    }
}

pub(crate) fn context(name: &str) -> FeedContext {
    FeedContext::for_session(name, SessionGeneration::initial())
}

pub(crate) fn request() -> DiscoveryRequest {
    DiscoveryRequest::default()
}

/// A playable kind-1 video note pinned to `created_at`, for cursor math.
pub(crate) fn note_at(created_at: u64) -> Event {
    EventBuilder::new(Kind::TextNote, "https://cdn.example/clip.mp4")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed test event")
}
