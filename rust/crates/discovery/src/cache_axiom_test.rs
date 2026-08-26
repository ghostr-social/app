use super::*;

use nostr_sdk::Client;

impl EventCache {
    /// Uses a private pool: the client's seen-ID database is deliberately
    /// eventless so late relay work cannot mutate account cache state.
    pub(in super::super) fn of(_client: &Client) -> Self {
        Self::session()
    }
    /// One query's answer: everything the relays streamed, in arrival
    /// order, plus the rows this session already holds for the same
    /// filter and the relays did not repeat. An empty pool changes
    /// nothing, so a cold query behaves exactly as it did before.
    pub(in super::super) async fn union(&self, filter: &Filter, fetched: Vec<Event>) -> Vec<Event> {
        self.union_for(SessionGeneration::initial(), filter, fetched)
            .await
            .unwrap_or_default()
    }
    /// Scopes the pool to one viewer and reports whether it emptied it.
    /// The engine outlives a sign-out — the gateway and its client are
    /// installed once per process — so a session that changes identity
    /// must not answer from the previous viewer's rows.
    pub(in super::super) async fn adopt(&self, viewer: ViewerScope) -> bool {
        self.adopt_for(SessionGeneration::initial(), viewer)
            .await
            .unwrap_or(false)
    }
}
