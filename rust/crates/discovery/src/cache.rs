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
        let session = self.generation().await;
        self.union_for(session, filter, fetched)
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
        self.write(&fetched).await;
        session.admit(&fetched);
        Some(merged(fetched, stored))
    }

    /// Everything stored for one filter, newest first, capped by that
    /// filter's own `limit` exactly as a relay caps its own answer.
    pub async fn stored(&self, filter: &Filter) -> Vec<Event> {
        let generation = self.generation().await;
        self.stored_for(generation, filter)
            .await
            .unwrap_or_default()
    }

    pub(crate) async fn stored_for(
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
            Ok(events) => events.to_vec(),
            Err(error) => {
                warn!("The session event pool could not be read: {error}");
                Vec::new()
            }
        }
    }

    /// Files an answer in the pool. Rejections (duplicate, replaced,
    /// ephemeral) are ordinary results, not errors.
    pub async fn remember(&self, events: &[Event]) {
        let generation = self.generation().await;
        self.remember_for(generation, events).await;
    }

    pub async fn remember_for(&self, generation: SessionGeneration, events: &[Event]) -> bool {
        let mut session = self.session.lock().await;
        if !session.matches(generation) {
            return false;
        }
        self.write(events).await;
        session.admit(events);
        true
    }

    async fn write(&self, events: &[Event]) {
        for event in events {
            if let Err(error) = self.database.save_event(event).await {
                warn!("The session event pool could not store an event: {error}");
            }
        }
    }

    /// Scopes the pool to one viewer and reports whether it emptied it.
    /// The engine outlives a sign-out — the gateway and its client are
    /// installed once per process — so a session that changes identity
    /// must not answer from the previous viewer's rows.
    #[cfg(test)]
    pub(crate) async fn adopt(&self, viewer: ViewerScope) -> bool {
        let generation = self.generation().await;
        self.adopt_for(generation, viewer).await.unwrap_or(false)
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

    async fn generation(&self) -> SessionGeneration {
        self.session.lock().await.generation()
    }

    async fn wipe(&self) {
        if let Err(error) = self.database.wipe().await {
            warn!("The session event pool could not be cleared: {error}");
        }
    }
}
