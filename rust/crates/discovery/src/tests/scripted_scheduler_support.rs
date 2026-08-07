use crate::discovery_scheduler::{
    start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig, RetrievalOutcome,
};
use crate::plan_executor::{PlanExecutor, PlanFailure, PlanFuture, PlannedRetrieval};
use ghostr_engine::{inventory_controller::Mode, DataUsageLevel};
use nostr_sdk::Event;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};

struct ScriptedExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    pages: Mutex<VecDeque<Result<Vec<Event>, PlanFailure>>>,
}

impl PlanExecutor for ScriptedExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let page = self.pages.lock().expect("pages").pop_front();
        Box::pin(async move {
            match page {
                Some(result) => result,
                None => std::future::pending().await,
            }
        })
    }
}

pub struct ScriptedScheduler {
    pub handle: DiscoveryHandle,
    pub started: mpsc::UnboundedReceiver<PlannedRetrieval>,
    pub outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
}

pub fn scripted_scheduler(pages: Vec<Vec<Event>>) -> ScriptedScheduler {
    scripted_scheduler_results(pages.into_iter().map(Ok).collect())
}

pub fn scripted_scheduler_results(
    pages: Vec<Result<Vec<Event>, PlanFailure>>,
) -> ScriptedScheduler {
    let (starts, started) = mpsc::unbounded_channel();
    let executor = Arc::new(ScriptedExecutor {
        starts,
        pages: Mutex::new(pages.into()),
    });
    let (outcome_sender, outcomes) = mpsc::unbounded_channel();
    let (_, modes) = watch::channel(Mode::Comfort);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor,
        level: DataUsageLevel::Conservative,
        modes,
        outcomes: outcome_sender,
    });
    ScriptedScheduler {
        handle,
        started,
        outcomes,
    }
}
