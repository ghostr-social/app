//! Shared fakes and builders for discovery scheduler tests: a gated
//! executor stands in for relay IO so no test ever touches the network.

use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, PlanFailure, RetrievalOutcome};
use crate::scheduler::{start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig};
use ghostr_engine::inventory_controller::Mode;
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
        let gate = self.gate.clone();
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
    pub(crate) handle: DiscoveryHandle,
    pub(crate) started: mpsc::UnboundedReceiver<PlannedRetrieval>,
    pub(crate) gate: Arc<Semaphore>,
    pub(crate) outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
    pub(crate) modes: watch::Sender<Mode>,
}

/// Boots a scheduler over a gated executor. The mode watch starts in
/// Comfort so a test-sent Hunger is a real transition.
pub(crate) fn start_scheduler(level: DataUsageLevel, events: Vec<Event>) -> SchedulerHarness {
    let (starts, started) = mpsc::unbounded_channel();
    let gate = Arc::new(Semaphore::new(0));
    let executor = Arc::new(GatedExecutor {
        starts,
        gate: gate.clone(),
        events,
    });
    let (outcome_sender, outcomes) = mpsc::unbounded_channel();
    let (modes, mode_updates) = watch::channel(Mode::Comfort);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor,
        level,
        modes: mode_updates,
        outcomes: outcome_sender,
    });
    SchedulerHarness {
        handle,
        started,
        gate,
        outcomes,
        modes,
    }
}

pub(crate) fn context(name: &str) -> FeedContext {
    FeedContext::new(name)
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
