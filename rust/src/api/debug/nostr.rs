//! Debug-only bootstrap that feeds native Nostr discovery into delivery.

use crate::api::engine_control::validated_relay_urls;
use crate::api::feed::mapping::parse_feed_id;
use crate::api::feed::projection::project;
use crate::api::runtime::discovery::{lock, DiscoveryBoot, DiscoveryRuntime, SharedFeedState};
use crate::api::feed_types::{FfiFeedPost, FfiFeedStage};
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::feed::store::FeedId;
use crate::engine::inventory_controller::Mode;
use crate::engine::VideoMeta;
use ghostr_delivery::debug::feed::{DebugFeed, DebugFeedItem, DebugFeedStage};
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;

const READ_RELAYS: &[&str] = &[
    "wss://relay.damus.io",
    "wss://relay.snort.social",
    "wss://relay.primal.net",
    "wss://nos.lol",
];
const SEARCH_RELAYS: &[&str] = &[
    "wss://relay.nostr.band",
    "wss://nostr.wine",
    "wss://relay.noswhere.com",
    "wss://search.nos.today",
    "wss://antiprimal.net",
    "wss://relay.ditto.pub",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugNostrConfiguration {
    pub read_relays: Vec<String>,
    pub search_relays: Vec<String>,
}

impl Default for DebugNostrConfiguration {
    fn default() -> Self {
        Self {
            read_relays: relay_strings(READ_RELAYS),
            search_relays: relay_strings(SEARCH_RELAYS),
        }
    }
}

impl DebugNostrConfiguration {
    pub fn from_environment() -> anyhow::Result<Self> {
        let defaults = Self::default();
        Ok(Self {
            read_relays: environment_relays("GHOSTR_NOSTR_RELAYS", defaults.read_relays)?,
            search_relays: environment_relays(
                "GHOSTR_NOSTR_SEARCH_RELAYS",
                defaults.search_relays,
            )?,
        })
    }
}

pub struct DebugNostrRuntime {
    _discovery: DiscoveryRuntime,
    feed_sync: JoinHandle<()>,
    relay_sync: JoinHandle<()>,
}

impl DebugNostrRuntime {
    pub async fn start(
        client: Arc<Client>,
        modes: watch::Receiver<Mode>,
        configuration: DebugNostrConfiguration,
        feed: DebugFeed,
    ) -> anyhow::Result<Self> {
        let relay_client = client.clone();
        let monitored_relays = configuration.read_relays.clone();
        let discovery = DiscoveryRuntime::start(DiscoveryBoot {
            client,
            modes,
            bootstrap: configuration.read_relays,
            search_relays: configuration.search_relays,
            candidates: Some(feed.delivery()),
        })
        .await;
        let session = discovery.feed_session(None).await?;
        let id = discovery
            .open_feed(FeedSpec::MainFeed { viewer: None }, None, session)
            .await?;
        let feed_id = parse_feed_id(&id)?;
        let (state, revisions) = discovery.watch_inputs(feed_id)?;
        let feed_sync = tokio::spawn(sync_feed(feed.clone(), state, feed_id, revisions));
        let relay_sync = tokio::spawn(crate::api::debug::relay_status::monitor(
            relay_client,
            feed,
            monitored_relays,
        ));
        Ok(Self {
            _discovery: discovery,
            feed_sync,
            relay_sync,
        })
    }
}

impl Drop for DebugNostrRuntime {
    fn drop(&mut self) {
        self.feed_sync.abort();
        self.relay_sync.abort();
    }
}

async fn sync_feed(
    feed: DebugFeed,
    state: SharedFeedState,
    feed_id: FeedId,
    mut revisions: watch::Receiver<u64>,
) {
    loop {
        let revision = *revisions.borrow_and_update();
        publish_revision(&feed, &state, feed_id, revision);
        if revisions.changed().await.is_err() {
            return;
        }
    }
}

fn publish_revision(feed: &DebugFeed, state: &SharedFeedState, id: FeedId, revision: u64) {
    let state = lock(state);
    let projection = project(&state, id);
    let stage = debug_stage(projection.stage);
    let items = projection
        .posts
        .into_iter()
        .filter_map(debug_item)
        .collect();
    feed.publish(revision, stage, items);
}

pub(crate) fn debug_item(post: FfiFeedPost) -> Option<DebugFeedItem> {
    let creator = creator_name(&post);
    let title = display_title(&post);
    Some(DebugFeedItem {
        id: post.post_id,
        event_id: post.event_id,
        title,
        creator,
        created_at: post.created_at,
        meta: VideoMeta {
            urls: post.media.urls,
            delivery: post.media.delivery.into(),
            sha256: post.media.sha256,
            size_bytes: post.media.size_bytes,
            duration_ms: post.media.duration_ms,
        },
    })
}

fn display_title(post: &FfiFeedPost) -> Option<String> {
    post.title.clone().or_else(|| {
        let caption = post.caption.trim();
        (!caption.is_empty()).then(|| caption.chars().take(80).collect())
    })
}

fn creator_name(post: &FfiFeedPost) -> String {
    if post.creator.display_name.trim().is_empty() {
        post.creator.handle.clone()
    } else {
        post.creator.display_name.clone()
    }
}

pub(crate) fn debug_stage(stage: FfiFeedStage) -> DebugFeedStage {
    match stage {
        FfiFeedStage::Loading => DebugFeedStage::Loading,
        FfiFeedStage::Settled => DebugFeedStage::Settled,
        FfiFeedStage::Failed => DebugFeedStage::Failed,
    }
}

pub(crate) fn environment_relays(key: &str, fallback: Vec<String>) -> anyhow::Result<Vec<String>> {
    let Ok(raw) = std::env::var(key) else {
        return Ok(fallback);
    };
    let relays = raw
        .split(',')
        .map(str::trim)
        .filter(|relay| !relay.is_empty())
        .map(str::to_owned)
        .collect();
    validated_relay_urls(relays)
}

fn relay_strings(relays: &[&str]) -> Vec<String> {
    relays.iter().map(|relay| (*relay).to_owned()).collect()
}
