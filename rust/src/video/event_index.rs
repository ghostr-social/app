use crate::video::event_identity::{
    canonical_event, canonical_native_videos, CanonicalEvent, CanonicalNativeVideo,
};
pub use crate::video::event_indexer::{run_event_identity_indexer, spawn_event_identity_indexer};
use crate::video::native_deletions::NativeDeletionTombstones;
use crate::video::native_models::NativeEventIdentity;
use nostr_sdk::{Event, Kind};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub const MAX_NATIVE_INVENTORY_ITEMS: usize = 128;

#[derive(Clone)]
pub struct NativeVideoIndex {
    capacity: usize,
    state: Arc<RwLock<NativeVideoIndexState>>,
}

#[derive(Default)]
struct NativeVideoIndexState {
    deletions: NativeDeletionTombstones,
    values: HashMap<String, CanonicalNativeVideo>,
    order: Vec<String>,
    revisions: HashMap<String, NativeEventIdentity>,
}

impl NativeVideoIndex {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            state: Arc::new(RwLock::new(NativeVideoIndexState::new(capacity.max(1)))),
        }
    }

    pub async fn record(&self, event: &Event) {
        if event.kind == Kind::EventDeletion {
            self.state.write().await.record_deletion(event);
            return;
        }
        let Some(revision) = canonical_event(event) else {
            return;
        };
        let items = canonical_native_videos(event);
        let mut state = self.state.write().await;
        if !state.accepts(&revision) {
            return;
        }
        if state
            .deletions
            .deletes(&revision.identity, &revision.coordinate)
        {
            return;
        }
        state.remove_coordinate(&revision.coordinate);
        state.remember_revision(revision);
        state.insert(items);
        state.trim(self.capacity);
    }

    pub async fn insert(&self, item: CanonicalNativeVideo) {
        let mut state = self.state.write().await;
        state.remove_coordinate(&item.coordinate);
        state.remember_revision(CanonicalEvent {
            coordinate: item.coordinate.clone(),
            identity: item.identity.clone(),
        });
        state.insert([item]);
        state.trim(self.capacity);
    }

    pub async fn ordered_videos(&self) -> Vec<CanonicalNativeVideo> {
        let state = self.state.read().await;
        state
            .order
            .iter()
            .filter_map(|id| state.values.get(id).cloned())
            .collect()
    }

    pub async fn ordered_ids(&self) -> Vec<String> {
        self.state.read().await.order.clone()
    }
}

pub fn new_native_video_index() -> NativeVideoIndex {
    NativeVideoIndex::new(MAX_NATIVE_INVENTORY_ITEMS)
}

impl NativeVideoIndexState {
    fn new(capacity: usize) -> Self {
        Self {
            deletions: NativeDeletionTombstones::new(capacity.saturating_mul(2)),
            ..Self::default()
        }
    }

    fn record_deletion(&mut self, event: &Event) {
        self.deletions.record(event);
        self.values
            .retain(|_, item| !self.deletions.deletes(&item.identity, &item.coordinate));
        self.order.retain(|id| self.values.contains_key(id));
    }

    fn accepts(&self, incoming: &CanonicalEvent) -> bool {
        let retained = self
            .values
            .values()
            .find(|item| item.coordinate == incoming.coordinate)
            .map(|item| &item.identity);
        self.revisions
            .get(&incoming.coordinate)
            .or(retained)
            .is_none_or(|current| is_newer(&incoming.identity, current))
    }

    fn remember_revision(&mut self, incoming: CanonicalEvent) {
        self.revisions
            .insert(incoming.coordinate, incoming.identity);
    }

    fn remove_coordinate(&mut self, coordinate: &str) {
        self.values.retain(|_, item| item.coordinate != coordinate);
        self.order.retain(|id| self.values.contains_key(id));
    }

    fn insert<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = CanonicalNativeVideo>,
    {
        for item in items {
            let id = item.inventory_id.clone();
            if self.values.insert(id.clone(), item).is_none() {
                self.order.push(id);
            }
        }
    }

    fn trim(&mut self, capacity: usize) {
        self.sort_newest_first();
        self.order.truncate(capacity);
        self.values.retain(|id, _| self.order.contains(id));
        self.trim_revisions(capacity);
    }

    fn sort_newest_first(&mut self) {
        let values = &self.values;
        self.order.sort_by(|left, right| {
            let left = values.get(left).expect("indexed native video");
            let right = values.get(right).expect("indexed native video");
            compare_newest(left, right)
        });
    }

    fn trim_revisions(&mut self, capacity: usize) {
        let mut coordinates = self.revisions.keys().cloned().collect::<Vec<_>>();
        coordinates
            .sort_by(|left, right| compare_identity(&self.revisions[right], &self.revisions[left]));
        coordinates.truncate(capacity);
        self.revisions.retain(|key, _| coordinates.contains(key));
    }
}

fn compare_newest(left: &CanonicalNativeVideo, right: &CanonicalNativeVideo) -> Ordering {
    right
        .identity
        .created_at
        .cmp(&left.identity.created_at)
        .then_with(|| left.identity.event_id.cmp(&right.identity.event_id))
}

fn compare_identity(left: &NativeEventIdentity, right: &NativeEventIdentity) -> Ordering {
    left.created_at
        .cmp(&right.created_at)
        .then_with(|| right.event_id.cmp(&left.event_id))
}

fn is_newer(incoming: &NativeEventIdentity, current: &NativeEventIdentity) -> bool {
    compare_identity(incoming, current).is_gt()
}

pub async fn index_event(event: &Event, index: &NativeVideoIndex) {
    index.record(event).await;
}
