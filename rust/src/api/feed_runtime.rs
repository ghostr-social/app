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
use crate::discovery::outbox_bootstrap::OutboxBootstrap;
use crate::discovery::outbox_directory::OutboxDirectory;
use crate::discovery::relay_plan_executor::{RelayPlanExecutor, SharedOutboxDirectory};
use crate::discovery::search_queries::SEARCH_RELAY_URLS;
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use flutter_rust_bridge::frb;
use nostr_sdk::{Client, Event, Timestamp};
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
    executor: RelayPlanExecutor,
    bootstrap: Arc<OutboxBootstrap>,
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
            executor: Arc::new(executor.clone()),
            level: DataUsageLevel::Balanced,
            modes: boot.modes,
            outcomes: sender.clone(),
        });
        let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
        let bootstrap = Arc::new(OutboxBootstrap::new(
            Arc::new(executor.clone()),
            outbox.clone(),
            sender,
        ));
        tokio::spawn(pump_outcomes(
            OutcomeSinks { state: state.clone(), bootstrap: bootstrap.clone() },
            outcomes,
        ));
        Self { handle, state, client: boot.client, outbox, executor, bootstrap }
    }

    /// Opens the feed, starts its first-page queries, and returns the
    /// handle Dart uses for every later call. The page leaves first and
    /// the relay-list chase follows it: NIP-65 routing improves the
    /// pages after it, never delays this one.
    pub(crate) fn open_feed(&self, spec: FeedSpec) -> String {
        let (feed, dispatch) = lock(&self.state).open(spec.clone());
        if let Some(open) = dispatch {
            self.handle.open_feed(open.context, open.request);
        }
        self.chase_relay_lists(&spec);
        feed.0.to_string()
    }

    /// Whose NIP-65 lists this feed needs: the viewer's own lists (which
    /// bring their follows), or the creators a profile grid just opened.
    fn chase_relay_lists(&self, spec: &FeedSpec) {
        match spec {
            FeedSpec::MainFeed { viewer: Some(viewer) } => self.bootstrap.viewer(*viewer),
            FeedSpec::Profile(creators) => self.bootstrap.authors(creators),
            _ => {}
        }
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

    /// Both knobs the level moves: the worker pool in the scheduler and
    /// the outbox fan-out in the executor.
    pub(crate) fn set_data_usage(&self, level: DataUsageLevel) {
        self.handle.set_data_usage(level);
        self.executor.set_data_usage(level);
    }
}

/// Where a retrieval's events land: feed rows in the state, relay lists
/// in the outbox directory.
#[frb(ignore)]
pub(crate) struct OutcomeSinks {
    pub(crate) state: SharedFeedState,
    pub(crate) bootstrap: Arc<OutboxBootstrap>,
}

/// Feeds every retrieval outcome into the feed state until the
/// scheduler ends (all handles dropped). Relay lists and the viewer's
/// own lists are filed on the way through, whether they arrived on a
/// feed page or on a bootstrap retrieval.
pub(crate) async fn pump_outcomes(
    sinks: OutcomeSinks,
    mut outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
) {
    while let Some(outcome) = outcomes.recv().await {
        if let Ok(events) = &outcome.result {
            file_lists(&sinks, events).await;
        }
        lock(&sinks.state).apply(&outcome.context, outcome.result);
    }
}

/// A replaced follow set re-routes the main feed and sends the
/// bootstrap after the new follows' relay lists.
async fn file_lists(sinks: &OutcomeSinks, events: &[Event]) {
    sinks.bootstrap.ingest(events).await;
    let follows = lock(&sinks.state).ingest_social(events);
    if let Some(follows) = follows {
        sinks.bootstrap.track_follows(follows).await;
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
