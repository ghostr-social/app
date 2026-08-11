//! Shared Nostr-feed state for the embedded delivery debugger.

mod clearing;
mod hls;
mod window;

use crate::delivery_events::DeliveryHandle;
use ghostr_engine::VideoMeta;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use window::delivery_focus;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DebugFeedStage {
    Loading,
    Settled,
    Failed,
}

#[derive(Clone, Debug)]
pub struct DebugFeedItem {
    pub id: String,
    pub event_id: String,
    pub title: Option<String>,
    pub creator: String,
    pub created_at: u64,
    pub meta: VideoMeta,
}

#[derive(Clone, Debug, Serialize)]
pub struct DebugFeedSnapshot {
    pub stage: DebugFeedStage,
    pub revision: u64,
    pub relays: Vec<DebugRelaySnapshot>,
    pub discovered_count: usize,
    pub current_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DebugRelaySnapshot {
    pub url: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DebugFeedMetadata {
    pub event_id: String,
    pub title: Option<String>,
    pub creator: String,
    pub created_at: u64,
    pub feed_index: usize,
    pub focus_distance: i64,
}

#[derive(Clone, Debug)]
pub struct DebugFeed {
    delivery: DeliveryHandle,
    state: Arc<RwLock<FeedState>>,
}

#[derive(Debug)]
struct FeedState {
    stage: DebugFeedStage,
    revision: u64,
    relays: Vec<DebugRelaySnapshot>,
    items: Vec<DebugFeedItem>,
    hidden_events: HashSet<String>,
    current_id: Option<String>,
}

impl DebugFeed {
    pub fn new(delivery: DeliveryHandle, relays: Vec<String>) -> Self {
        Self {
            delivery,
            state: Arc::new(RwLock::new(FeedState {
                stage: DebugFeedStage::Loading,
                revision: 0,
                relays: relays.into_iter().map(initial_relay).collect(),
                items: Vec::new(),
                hidden_events: HashSet::new(),
                current_id: None,
            })),
        }
    }

    pub fn publish(&self, revision: u64, stage: DebugFeedStage, mut items: Vec<DebugFeedItem>) {
        let focus = {
            let mut state = self.write();
            items.retain(|item| !state.hidden_events.contains(&item.event_id));
            state.revision = revision;
            state.stage = stage;
            state.current_id = retained_current(&state.current_id, &items);
            state.items = items;
            delivery_focus(&state)
        };
        self.delivery.update_focus(focus);
    }

    pub fn select(&self, id: &str) -> anyhow::Result<()> {
        let focus = {
            let mut state = self.write();
            anyhow::ensure!(
                state.items.iter().any(|item| item.id == id),
                "video is not in the Nostr feed"
            );
            state.current_id = Some(id.to_owned());
            delivery_focus(&state)
        };
        self.delivery.update_focus(focus);
        Ok(())
    }

    pub fn update_relays(&self, relays: Vec<DebugRelaySnapshot>) {
        self.write().relays = relays;
    }

    pub fn delivery(&self) -> DeliveryHandle {
        self.delivery.clone()
    }

    pub fn snapshot(&self) -> DebugFeedSnapshot {
        let state = self.read();
        DebugFeedSnapshot {
            stage: state.stage,
            revision: state.revision,
            relays: state.relays.clone(),
            discovered_count: state.items.len(),
            current_id: state.current_id.clone(),
        }
    }

    pub fn metadata(&self, id: &str) -> Option<DebugFeedMetadata> {
        let state = self.read();
        let index = state.items.iter().position(|item| item.id == id)?;
        Some(metadata(&state, index))
    }

    fn read(&self) -> RwLockReadGuard<'_, FeedState> {
        self.state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, FeedState> {
        self.state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn retained_current(current: &Option<String>, items: &[DebugFeedItem]) -> Option<String> {
    current
        .as_ref()
        .filter(|id| items.iter().any(|item| item.id == id.as_str()))
        .cloned()
        .or_else(|| items.first().map(|item| item.id.clone()))
}

fn metadata(state: &FeedState, index: usize) -> DebugFeedMetadata {
    let item = &state.items[index];
    let current = state
        .current_id
        .as_ref()
        .and_then(|id| state.items.iter().position(|item| &item.id == id))
        .unwrap_or(0);
    DebugFeedMetadata {
        event_id: item.event_id.clone(),
        title: item.title.clone(),
        creator: item.creator.clone(),
        created_at: item.created_at,
        feed_index: index,
        focus_distance: index as i64 - current as i64,
    }
}

fn initial_relay(url: String) -> DebugRelaySnapshot {
    DebugRelaySnapshot {
        url,
        status: "initializing".to_owned(),
    }
}
