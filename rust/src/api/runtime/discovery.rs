//! Boots Rust discovery next to the engine (plan §5.3–§5.5): one
//! scheduler over the shared Nostr client, one locked `FeedState`, and
//! one pump task feeding retrieval outcomes into it. Installed into
//! `EngineHandles` by `runtime_registry`.

use crate::api::feed::decisions::LoadMoreAction;
use crate::api::feed::state::FeedState;
use crate::discovery::execution::relay_executor::RelayPlanExecutor;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::feed::store::FeedId;
use crate::discovery::outbox::bootstrap::OutboxBootstrap;
use crate::discovery::outbox::directory::SharedOutboxDirectory;
use crate::discovery::relay::pool::RelayPoolOwner;
use crate::discovery::scheduler::DiscoveryHandle;
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use flutter_rust_bridge::frb;
use ghostr_delivery::delivery_events::DeliveryHandle;
use nostr_sdk::{Client, Timestamp};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::watch;

pub(crate) use crate::api::feed::outcome_pump::{pump_outcomes, OutcomeSinks};

/// The one `FeedState` behind a lock; stream watchers snapshot
/// through it while the pump task and FFI calls mutate it.
pub(crate) type SharedFeedState = Arc<Mutex<FeedState>>;

/// Everything discovery boot needs from the started gateway.
#[frb(ignore)]
pub(crate) struct DiscoveryBoot {
    pub client: Arc<Client>,
    pub modes: watch::Receiver<Mode>,
    pub bootstrap: Vec<String>,
    pub search_relays: Vec<String>,
    pub candidates: Option<DeliveryHandle>,
}

/// The FFI layer's grip on Rust discovery after a successful start.
#[frb(ignore)]
pub(crate) struct DiscoveryRuntime {
    pub(crate) handle: DiscoveryHandle,
    pub(crate) state: SharedFeedState,
    pub(crate) client: Arc<Client>,
    pub(crate) outbox: SharedOutboxDirectory,
    pub(crate) executor: RelayPlanExecutor,
    pub(crate) bootstrap: Arc<OutboxBootstrap>,
    pub(crate) relay_pool: Arc<RelayPoolOwner>,
}

impl DiscoveryRuntime {
    /// Starts the scheduler and the outcome pump. Discovery begins at
    /// the balanced data-usage level, like the delivery manager;
    /// `ffi_set_delivery_config` adjusts it live.
    pub(crate) async fn start(boot: DiscoveryBoot) -> Self {
        crate::api::runtime::start::start(boot).await
    }

    /// Opens the feed, starts its first-page queries, and returns the
    /// handle Dart uses for every later call. The page leaves first and
    /// the relay-list chase follows it: NIP-65 routing improves the
    /// pages after it, never delays this one.
    pub(crate) async fn open_feed(
        &self,
        spec: FeedSpec,
        expected_account: Option<nostr_sdk::PublicKey>,
        expected_session: crate::discovery::session_generation::SessionGeneration,
    ) -> anyhow::Result<String> {
        let _account_guard = self
            .relay_pool
            .begin_account_request(expected_account, expected_session)
            .await?;
        let (feed, dispatch) = lock(&self.state).open(spec.clone());
        if let Some(open) = dispatch {
            self.handle.open_feed(open.context, open.request);
        }
        self.chase_relay_lists(&spec);
        Ok(feed.0.to_string())
    }

    pub(crate) async fn feed_session(
        &self,
        expected_account: Option<nostr_sdk::PublicKey>,
    ) -> anyhow::Result<crate::discovery::session_generation::SessionGeneration> {
        self.relay_pool.account_session(expected_account).await
    }

    /// Whose NIP-65 lists this feed needs: the viewer's own lists (which
    /// bring their follows), or the creators a profile grid just opened.
    fn chase_relay_lists(&self, spec: &FeedSpec) {
        match spec {
            FeedSpec::MainFeed {
                viewer: Some(viewer),
            } => self.bootstrap.viewer(*viewer),
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
            LoadMoreAction::Older {
                context,
                older_than,
            } => {
                self.handle.load_more(context, Some(older_than));
            }
        }
        decision.may_have_more
    }

    pub(crate) fn close_feed(&self, feed: FeedId) {
        if let Some(context) = lock(&self.state).close(feed) {
            self.handle.close_feed(context);
        }
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

/// Locks the feed state, recovering from poisoning like the other
/// api-side registries.
pub(crate) fn lock(state: &SharedFeedState) -> MutexGuard<'_, FeedState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
