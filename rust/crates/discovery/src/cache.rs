//! The bounded account-session event pool merged into every relay answer.
//!
//! The SDK's relay stream does not read its database back into query
//! results, so stored rows are selected with the same filter here and
//! appended after fresh relay events. A private database plus generation
//! checks keeps late work from one account out of the next account's
//! answers. `MAX_CACHED_EVENTS` defines the in-memory bound.

pub mod database;
mod durable;
pub(crate) mod merge;
pub(crate) mod persistence;
pub mod session;

use log::warn;
use nostr_sdk::prelude::{Event, Filter, NostrDatabase};

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cache::database::MAX_CACHED_EVENTS;
pub use crate::cache::database::{client_with_event_cache, session_event_database};
use crate::cache::merge::merged;
use crate::cache::session::EventCacheSession;
pub use crate::cache::session::ViewerScope;
use crate::content::parsing::MAX_REPOSTABLE_EVENT_BYTES;
use crate::session_generation::SessionGeneration;

/// Read side of the client's database, scoped to one viewer.
pub struct EventCache {
    database: Arc<dyn NostrDatabase>,
    persistence: Option<persistence::EventCachePersistence>,
    session: Mutex<EventCacheSession>,
}

impl EventCache {
    pub(super) fn session() -> Self {
        Self::new(Arc::new(session_event_database(MAX_CACHED_EVENTS)))
    }

    pub fn new(database: Arc<dyn NostrDatabase>) -> Self {
        Self {
            database,
            persistence: None,
            session: Mutex::new(EventCacheSession::initial()),
        }
    }

    pub(super) async fn union_for(
        &self,
        generation: SessionGeneration,
        filter: &Filter,
        fetched: Vec<Event>,
    ) -> Option<Vec<Event>> {
        let mut session = self.session.lock().await;
        if !session.matches(generation) {
            return None;
        }
        let mut stored = self.read(filter).await;
        session.retain_admitted(&mut stored);
        let admitted = self.write(&fetched).await;
        session.admit(&admitted);
        self.persist_after_write(&session, &admitted).await;
        Some(merged(fetched, stored))
    }

    pub async fn stored_for(
        &self,
        generation: SessionGeneration,
        filter: &Filter,
    ) -> Option<Vec<Event>> {
        let session = self.session.lock().await;
        if !session.matches(generation) {
            return None;
        }
        let mut stored = self.read(filter).await;
        session.retain_admitted(&mut stored);
        Some(stored)
    }

    async fn read(&self, filter: &Filter) -> Vec<Event> {
        match self.database.query(vec![filter.clone()]).await {
            Ok(events) => events.into_iter().filter(cacheable_event).collect(),
            Err(error) => {
                warn!("The session event pool could not be read: {error}");
                Vec::new()
            }
        }
    }

    pub async fn remember_for(&self, generation: SessionGeneration, events: &[Event]) -> bool {
        let mut session = self.session.lock().await;
        if !session.matches(generation) {
            return false;
        }
        let admitted = self.write(events).await;
        session.admit(&admitted);
        self.persist_after_write(&session, &admitted).await;
        true
    }

    async fn write(&self, events: &[Event]) -> Vec<nostr_sdk::EventId> {
        let mut admitted = Vec::new();
        for event in events.iter().filter(|event| cacheable_event(event)) {
            match self.database.save_event(event).await {
                Ok(_) => admitted.push(event.id),
                Err(error) => warn!("The session event pool could not store an event: {error}"),
            }
        }
        admitted
    }

    pub(super) async fn adopt_for(
        &self,
        generation: SessionGeneration,
        viewer: ViewerScope,
    ) -> Option<bool> {
        let mut session = self.session.lock().await;
        if !session.matches(generation) {
            return None;
        }
        if viewer == ViewerScope::Unknown {
            return Some(false);
        }
        let previous = session.viewer();
        if previous == viewer {
            return Some(false);
        }
        let replaced = session.adopt(viewer);
        self.bind_viewer(&mut session, viewer, replaced).await;
        Some(replaced)
    }

    pub async fn reset_session(&self, generation: SessionGeneration) {
        let mut session = self.session.lock().await;
        session.reset(generation);
        self.wipe().await;
    }

    pub async fn reset_session_for(&self, generation: SessionGeneration, viewer: ViewerScope) {
        let mut session = self.session.lock().await;
        session.reset(generation);
        self.wipe().await;
        if viewer != ViewerScope::Unknown {
            session.adopt(viewer);
        }
        self.restore_bound_viewer(&mut session, viewer).await;
    }

    pub(super) async fn is_current(&self, generation: SessionGeneration) -> bool {
        self.session.lock().await.matches(generation)
    }

    async fn wipe(&self) {
        if let Err(error) = self.database.wipe().await {
            warn!("The session event pool could not be cleared: {error}");
        }
    }
}

fn cacheable_event(event: &Event) -> bool {
    !matches!(event.kind.as_u16(), 6 | 16) || event.content.len() <= MAX_REPOSTABLE_EVENT_BYTES
}

#[cfg(test)]
#[path = "cache_axiom_test.rs"]
mod axiom_test_support;
