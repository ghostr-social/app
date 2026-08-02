use crate::video::event_identity::{canonical_native_videos, CanonicalNativeVideo};
use nostr_sdk::{Client, Event, RelayPoolNotification};
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
    values: HashMap<String, CanonicalNativeVideo>,
    order: Vec<String>,
}

impl NativeVideoIndex {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            state: Arc::new(RwLock::new(NativeVideoIndexState::default())),
        }
    }

    pub async fn record(&self, event: &Event) {
        let items = canonical_native_videos(event);
        let Some(incoming) = items.first() else {
            return;
        };
        let mut state = self.state.write().await;
        if !state.accepts(incoming) {
            return;
        }
        state.remove_coordinate(&incoming.coordinate);
        state.insert(items);
        state.trim(self.capacity);
    }

    pub async fn insert(&self, item: CanonicalNativeVideo) {
        let mut state = self.state.write().await;
        state.remove_coordinate(&item.coordinate);
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
    fn accepts(&self, incoming: &CanonicalNativeVideo) -> bool {
        let existing = self
            .values
            .values()
            .find(|item| item.coordinate == incoming.coordinate);
        existing.is_none_or(|current| is_newer(incoming, current))
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
        while self.order.len() > capacity {
            let id = self.order.remove(0);
            self.values.remove(&id);
        }
    }
}

fn is_newer(incoming: &CanonicalNativeVideo, current: &CanonicalNativeVideo) -> bool {
    incoming.identity.created_at > current.identity.created_at
        || (incoming.identity.created_at == current.identity.created_at
            && incoming.identity.event_id < current.identity.event_id)
}

pub fn spawn_event_identity_indexer(client: Arc<Client>, index: NativeVideoIndex) {
    let mut notifications = client.notifications();
    tokio::spawn(async move {
        while let Ok(notification) = notifications.recv().await {
            let RelayPoolNotification::Event { event, .. } = notification else {
                continue;
            };
            index.record(&event).await;
        }
    });
}

pub async fn index_event(event: &Event, index: &NativeVideoIndex) {
    index.record(event).await;
}
