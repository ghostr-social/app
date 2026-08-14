//! The bounded account-session event pool merged into every relay answer.
//!
//! The SDK's relay stream does not read its database back into query
//! results, so stored rows are selected with the same filter here and
//! appended after fresh relay events. A private database plus generation
//! checks keeps late work from one account out of the next account's
//! answers. See [`MAX_CACHED_EVENTS`] for the in-memory bound.

pub mod database;
pub(crate) mod merge;
pub mod session;

use log::warn;
use nostr_sdk::prelude::{Event, Filter, NostrDatabase};
#[cfg(test)]
use nostr_sdk::Client;
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
    session: Mutex<EventCacheSession>,
}

impl EventCache {
    /// Uses a private pool: the client's seen-ID database is deliberately
    /// eventless so late relay work cannot mutate account cache state.
    #[cfg(test)]
    pub(crate) fn of(_client: &Client) -> Self {
        Self::session()
    }

    pub(crate) fn session() -> Self {
        Self::new(Arc::new(session_event_database(MAX_CACHED_EVENTS)))
    }

    pub fn new(database: Arc<dyn NostrDatabase>) -> Self {
        Self {
            database,
            session: Mutex::new(EventCacheSession::initial()),
        }
    }

    /// One query's answer: everything the relays streamed, in arrival
    /// order, plus the rows this session already holds for the same
    /// filter and the relays did not repeat. An empty pool changes
    /// nothing, so a cold query behaves exactly as it did before.
    #[cfg(test)]
    pub(crate) async fn union(&self, filter: &Filter, fetched: Vec<Event>) -> Vec<Event> {
        self.union_for(SessionGeneration::initial(), filter, fetched)
            .await
            .unwrap_or_default()
    }

    pub(crate) async fn union_for(
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

    /// Scopes the pool to one viewer and reports whether it emptied it.
    /// The engine outlives a sign-out — the gateway and its client are
    /// installed once per process — so a session that changes identity
    /// must not answer from the previous viewer's rows.
    #[cfg(test)]
    pub(crate) async fn adopt(&self, viewer: ViewerScope) -> bool {
        self.adopt_for(SessionGeneration::initial(), viewer)
            .await
            .unwrap_or(false)
    }

    pub(crate) async fn adopt_for(
        &self,
        generation: SessionGeneration,
        viewer: ViewerScope,
    ) -> Option<bool> {
        let mut session = self.session.lock().await;
        if !session.matches(generation) {
            return None;
        }
        if !session.adopt(viewer) {
            return Some(false);
        }
        self.wipe().await;
        Some(true)
    }

    pub async fn reset_session(&self, generation: SessionGeneration) {
        let mut session = self.session.lock().await;
        session.reset(generation);
        self.wipe().await;
    }

    pub(crate) async fn is_current(&self, generation: SessionGeneration) -> bool {
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
