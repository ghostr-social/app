//! Bounded, viewer-scoped persistence for the in-memory event pool.

use crate::cache::persistence::format::{decode, encode};
use crate::cache::persistence::storage::{clear, read, replace};
use crate::cache::session::ViewerScope;
use log::warn;
use nostr_sdk::Event;
use std::path::{Path, PathBuf};

mod format;
mod storage;

pub(crate) use format::MAX_SNAPSHOT_BYTES;

const SNAPSHOT_FILE: &str = "nostr-event-cache-v1.json";

pub(crate) fn snapshot_path(root: &Path) -> PathBuf {
    root.join(SNAPSHOT_FILE)
}

pub(crate) struct EventCachePersistence {
    path: PathBuf,
}

impl EventCachePersistence {
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            path: snapshot_path(root),
        }
    }

    pub(crate) async fn load(&self, viewer: ViewerScope) -> Vec<Event> {
        if viewer == ViewerScope::Unknown {
            return Vec::new();
        }
        let bytes = match read(&self.path).await {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Vec::new(),
            Err(error) => return self.reject(error).await,
        };
        match decode(&bytes, viewer) {
            Ok(events) => events,
            Err(error) => self.reject(error).await,
        }
    }

    pub(crate) async fn store(&self, viewer: ViewerScope, events: &[Event]) {
        if viewer == ViewerScope::Unknown {
            return;
        }
        let body = match encode(viewer, events) {
            Ok(body) => body,
            Err(error) => {
                warn!("The durable event pool could not be encoded: {error}");
                return;
            }
        };
        if let Err(error) = replace(&self.path, &body).await {
            warn!("The durable event pool could not be stored: {error}");
        }
    }

    async fn reject(&self, error: anyhow::Error) -> Vec<Event> {
        warn!("The durable event pool was rejected: {error}");
        self.clear().await;
        Vec::new()
    }

    async fn clear(&self) {
        if let Err(error) = clear(&self.path).await {
            warn!("The durable event pool could not be cleared: {error}");
        }
    }
}
