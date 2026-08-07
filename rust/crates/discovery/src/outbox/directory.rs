//! NIP-65 outbox routing: kind-10002 relay lists become per-author
//! write-relay lists so queries reach the relays where wanted authors
//! publish. Pure ingestion and lookup; the scheduler owns retrieval.

use crate::outbox::relay_list::write_urls;
use crate::session_generation::SessionGeneration;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Kind, PublicKey, Timestamp};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared live outbox directory; ingestion happens on the subscription side.
pub type SharedOutboxDirectory = Arc<RwLock<OutboxDirectory>>;

/// Outbox fan-out cap per data-usage level; mirrors `maxOutboxRelays`
/// in lib/features/settings/domain/data_usage_level.dart.
pub fn max_outbox_relays(level: DataUsageLevel) -> usize {
    match level {
        DataUsageLevel::Conservative => 6,
        DataUsageLevel::Balanced => 12,
        DataUsageLevel::Aggressive => 18,
    }
}

/// Per-author write relays resolved from ingested kind-10002 events,
/// merged with the configured bootstrap relays on lookup.
#[derive(Debug, Default)]
pub struct OutboxDirectory {
    bootstrap: Vec<String>,
    write_lists: HashMap<PublicKey, WriteRelayList>,
    viewer_follows: Vec<PublicKey>,
    session: SessionGeneration,
}

#[derive(Debug)]
struct WriteRelayList {
    created_at: Timestamp,
    urls: Vec<String>,
}

impl OutboxDirectory {
    /// Bootstrap relays lead every lookup result; pass them already
    /// validated (the Dart side hands over `RelayUrl` values).
    pub fn new(bootstrap_relays: Vec<String>) -> Self {
        Self {
            bootstrap: bootstrap_relays,
            write_lists: HashMap::new(),
            viewer_follows: Vec::new(),
            session: SessionGeneration::initial(),
        }
    }

    /// Drops account-derived routing while retaining configured relays.
    pub fn reset_session(&mut self, session: SessionGeneration) {
        self.session = session;
        self.write_lists.clear();
        self.viewer_follows.clear();
    }

    pub fn is_session(&self, session: SessionGeneration) -> bool {
        self.session == session
    }

    pub fn replace_bootstrap(&mut self, relays: Vec<String>) -> Vec<String> {
        std::mem::replace(&mut self.bootstrap, relays)
    }

    /// Ingests a kind-10002 relay list; anything else is ignored.
    /// Replaceable semantics: a strictly newer timestamp replaces the
    /// author's list; ties keep the accepted value.
    pub fn ingest(&mut self, event: &Event) {
        if event.kind != Kind::RelayList || !self.accepts(&event.pubkey, event.created_at) {
            return;
        }
        let list = WriteRelayList {
            created_at: event.created_at,
            urls: write_urls(event),
        };
        self.write_lists.insert(event.pubkey, list);
    }

    pub(crate) fn ingest_for(&mut self, session: SessionGeneration, event: &Event) {
        if self.session == session {
            self.ingest(event);
        }
    }

    /// Ingests a whole retrieval's events; anything that is not a
    /// kind-10002 relay list is ignored.
    pub fn ingest_all(&mut self, events: &[Event]) {
        for event in events {
            self.ingest(event);
        }
    }

    pub(crate) fn ingest_all_for(&mut self, session: SessionGeneration, events: &[Event]) {
        for event in events {
            self.ingest_for(session, event);
        }
    }

    /// Remembers the signed-in viewer's follows after the bootstrap task
    /// retrieves their kind-3 list. Later unscoped feed queries can then
    /// route through those authors' write relays.
    pub fn track_viewer_follows(&mut self, follows: Vec<PublicKey>) {
        self.viewer_follows = follows;
    }

    pub(crate) fn track_viewer_follows_for(
        &mut self,
        session: SessionGeneration,
        follows: Vec<PublicKey>,
    ) {
        if self.session == session {
            self.track_viewer_follows(follows);
        }
    }

    /// `discoveryRelayUrls`: where the viewer's follows publish. Queries
    /// route here without filtering by the follows, so the relays answer
    /// with everything they carry.
    pub fn discovery_relays(&self, cap: usize) -> Vec<String> {
        self.relays_for_authors(&self.viewer_follows, cap)
    }

    /// The author's declared write relays, in declaration order.
    pub fn write_relays(&self, author: &PublicKey) -> &[String] {
        self.write_lists
            .get(author)
            .map_or(&[], |list| list.urls.as_slice())
    }

    /// Relays where the given authors publish: ranked by how many of
    /// the (deduplicated) authors write there, ties by URL ascending,
    /// capped at `cap`, then merged after the bootstrap relays.
    /// Authors without a known relay list contribute nothing, so an
    /// all-unknown query falls back to the bootstrap set.
    pub fn relays_for_authors(&self, authors: &[PublicKey], cap: usize) -> Vec<String> {
        merged(&self.bootstrap, self.ranked_write_relays(authors, cap))
    }

    fn accepts(&self, author: &PublicKey, created_at: Timestamp) -> bool {
        self.write_lists
            .get(author)
            .is_none_or(|list| created_at > list.created_at)
    }

    fn ranked_write_relays(&self, authors: &[PublicKey], cap: usize) -> Vec<String> {
        let unique: HashSet<&PublicKey> = authors.iter().collect();
        let mut counts: HashMap<&String, usize> = HashMap::new();
        for author in unique {
            for url in self.write_relays(author) {
                *counts.entry(url).or_default() += 1;
            }
        }
        let mut ranked: Vec<&String> = counts.keys().copied().collect();
        ranked.sort_by(|left, right| {
            counts[right]
                .cmp(&counts[left])
                .then_with(|| left.cmp(right))
        });
        ranked.truncate(cap);
        ranked.into_iter().cloned().collect()
    }
}

fn merged(bootstrap: &[String], outbox: Vec<String>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();
    for url in bootstrap.iter().cloned().chain(outbox) {
        if seen.insert(url.clone()) {
            result.push(url);
        }
    }
    result
}
