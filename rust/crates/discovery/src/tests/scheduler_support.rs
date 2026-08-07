//! Shared fakes and builders for discovery scheduler tests: a gated
//! executor stands in for relay IO so no test ever touches the network.

use crate::discovery_scheduler::{
    start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig,
};
use crate::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::retrieval_types::{FeedContext, PlanFailure, RetrievalOutcome};
use crate::video_filters::DiscoveryRequest;
use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Semaphore};

pub use super::scheduler_wait::{next_outcome, next_started, no_start};

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

pub struct SchedulerHarness {
    pub handle: DiscoveryHandle,
    pub started: mpsc::UnboundedReceiver<PlannedRetrieval>,
    pub gate: Arc<Semaphore>,
    pub outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
    pub modes: watch::Sender<Mode>,
}

/// Boots a scheduler over a gated executor. The mode watch starts in
/// Comfort so a test-sent Hunger is a real transition.
pub fn start_scheduler(level: DataUsageLevel, events: Vec<Event>) -> SchedulerHarness {
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

pub fn context(name: &str) -> FeedContext {
    FeedContext::new(name)
}

pub fn request() -> DiscoveryRequest {
    DiscoveryRequest::default()
}

/// A playable kind-1 video note pinned to `created_at`, for cursor math.
pub fn note_at(created_at: u64) -> Event {
    EventBuilder::new(Kind::TextNote, "https://cdn.example/clip.mp4")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed test event")
}
