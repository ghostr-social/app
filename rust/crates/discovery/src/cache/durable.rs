//! Durable lifecycle layered onto the canonical in-memory event pool.

use super::database::{session_event_database, MAX_CACHED_EVENTS};
use super::persistence::EventCachePersistence;
use super::session::{EventCacheSession, ViewerScope};
use super::EventCache;
use nostr_sdk::{Event, EventId, Filter};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

impl EventCache {
    pub fn persistent(root: impl AsRef<Path>) -> Self {
        Self {
            database: Arc::new(session_event_database(MAX_CACHED_EVENTS)),
            persistence: Some(EventCachePersistence::new(root.as_ref())),
            session: Mutex::new(EventCacheSession::initial()),
        }
    }

    pub(super) async fn bind_viewer(
        &self,
        session: &mut EventCacheSession,
        viewer: ViewerScope,
        replaced: bool,
    ) {
        if self.persistence.is_some() {
            if replaced {
                self.wipe().await;
            }
            self.restore_bound_viewer(session, viewer).await;
        } else if replaced {
            self.wipe().await;
        }
    }

    pub(super) async fn restore_bound_viewer(
        &self,
        session: &mut EventCacheSession,
        viewer: ViewerScope,
    ) {
        let Some(persistence) = &self.persistence else {
            return;
        };
        let restored = persistence.load(viewer).await;
        let restored_ids = self.write(&restored).await;
        session.admit(&restored_ids);
        let mut events = self.read_all().await;
        session.retain_admitted(&mut events);
        let admitted: Vec<EventId> = events.iter().map(|event| event.id).collect();
        session.restore(&admitted);
        persistence.store(viewer, &events).await;
    }

    pub(super) async fn persist_after_write(
        &self,
        session: &EventCacheSession,
        admitted: &[EventId],
    ) {
        if admitted.is_empty() {
            return;
        }
        let Some(persistence) = &self.persistence else {
            return;
        };
        let mut events = self.read_all().await;
        session.retain_admitted(&mut events);
        persistence.store(session.viewer(), &events).await;
    }

    async fn read_all(&self) -> Vec<Event> {
        self.read(&Filter::new().limit(MAX_CACHED_EVENTS)).await
    }
}
