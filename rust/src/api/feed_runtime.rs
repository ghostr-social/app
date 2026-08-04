//! Boots Rust discovery next to the engine (plan §5.3–§5.5): one
//! scheduler over the shared Nostr client, one locked `FeedState`, and
//! one pump task feeding retrieval outcomes into it. Installed into
//! `EngineHandles` by `runtime_registry`.

use crate::api::feed_decisions::LoadMoreAction;
use crate::api::feed_state::FeedState;
use crate::discovery::discovery_scheduler::{
    start_discovery_scheduler, DiscoveryHandle, DiscoverySchedulerConfig, RetrievalOutcome,
};
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::feed_store::FeedId;
use crate::discovery::outbox_directory::OutboxDirectory;
use crate::discovery::relay_plan_executor::{RelayPlanExecutor, SharedOutboxDirectory};
use crate::discovery::search_queries::SEARCH_RELAY_URLS;
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use flutter_rust_bridge::frb;
use nostr_sdk::{Client, Timestamp};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::{mpsc, watch, RwLock};

/// The one `FeedState` behind a lock; stream watchers snapshot
/// through it while the pump task and FFI calls mutate it.
pub(crate) type SharedFeedState = Arc<Mutex<FeedState>>;

/// Everything discovery boot needs from the started gateway.
#[frb(ignore)]
pub(crate) struct DiscoveryBoot {
    pub client: Arc<Client>,
    pub modes: watch::Receiver<Mode>,
    pub bootstrap: Vec<String>,
}

/// The FFI layer's grip on Rust discovery after a successful start.
#[frb(ignore)]
pub(crate) struct DiscoveryRuntime {
    pub(crate) handle: DiscoveryHandle,
    pub(crate) state: SharedFeedState,
    pub(crate) client: Arc<Client>,
    pub(crate) outbox: SharedOutboxDirectory,
}

impl DiscoveryRuntime {
    /// Starts the scheduler and the outcome pump. Discovery begins at
    /// the balanced data-usage level, like the delivery manager;
    /// `ffi_set_delivery_config` adjusts it live.
    pub(crate) fn start(boot: DiscoveryBoot) -> Self {
        let outbox: SharedOutboxDirectory =
            Arc::new(RwLock::new(OutboxDirectory::new(boot.bootstrap)));
        let executor = RelayPlanExecutor::new(
            boot.client.clone(),
            search_relays(),
            outbox.clone(),
            DataUsageLevel::Balanced,
        );
        let (sender, outcomes) = mpsc::unbounded_channel();
        let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
            executor: Arc::new(executor),
            level: DataUsageLevel::Balanced,
            modes: boot.modes,
            outcomes: sender,
        });
        let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
        tokio::spawn(pump_outcomes(state.clone(), outcomes));
        Self { handle, state, client: boot.client, outbox }
    }

    /// Opens the feed, starts its first-page queries, and returns the
    /// handle Dart uses for every later call.
    pub(crate) fn open_feed(&self, spec: FeedSpec) -> String {
        let (feed, dispatch) = lock(&self.state).open(spec);
        if let Some(open) = dispatch {
            self.handle.open_feed(open.context, open.request);
        }
        feed.0.to_string()
    }

    /// Claims and dispatches one older page; the return value reports
    /// whether more content may exist.
    pub(crate) fn load_more(&self, feed: FeedId, explicit: Option<Timestamp>) -> bool {
        let decision = lock(&self.state).load_more(feed, explicit);
        match decision.action {
            LoadMoreAction::None => {}
            LoadMoreAction::Reopen(open) => self.handle.open_feed(open.context, open.request),
            LoadMoreAction::Older { context, older_than } => {
                self.handle.load_more(context, Some(older_than));
            }
        }
        decision.may_have_more
    }

    pub(crate) fn close_feed(&self, feed: FeedId) {
        lock(&self.state).close(feed);
    }

    /// The state handle plus a revision watch for one stream watcher.
    pub(crate) fn watch_inputs(
        &self,
        feed: FeedId,
    ) -> anyhow::Result<(SharedFeedState, watch::Receiver<u64>)> {
        let revisions = lock(&self.state)
            .subscribe(feed)
            .ok_or_else(|| anyhow::anyhow!("the feed is not open"))?;
        Ok((self.state.clone(), revisions))
    }

    pub(crate) fn set_data_usage(&self, level: DataUsageLevel) {
        self.handle.set_data_usage(level);
    }
}

/// Feeds every retrieval outcome into the feed state until the
/// scheduler ends (all handles dropped).
pub(crate) async fn pump_outcomes(
    state: SharedFeedState,
    mut outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
) {
    while let Some(outcome) = outcomes.recv().await {
        lock(&state).apply(&outcome.context, outcome.result);
    }
}

/// Locks the feed state, recovering from poisoning like the other
/// api-side registries.
pub(crate) fn lock(state: &SharedFeedState) -> MutexGuard<'_, FeedState> {
    state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn search_relays() -> Vec<String> {
    SEARCH_RELAY_URLS.iter().map(|url| (*url).to_owned()).collect()
}
