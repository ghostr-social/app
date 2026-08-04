//! NIP-65 outbox routing (plan §5.2): kind-10002 relay lists become
//! per-author write-relay lists so queries reach the relays where the
//! wanted authors actually publish. Pure ingestion and lookup — the
//! scheduler owns subscriptions and refresh. Mirrors
//! lib/platform/nostr/ndk_nostr_outbox_directory.dart.

use crate::discovery::relay_url::normalize_relay_url;
use crate::engine::DataUsageLevel;
use nostr_sdk::{Event, Kind, PublicKey, Timestamp};
use std::collections::{HashMap, HashSet};

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
        }
    }

    /// Ingests a kind-10002 relay list; anything else is ignored.
    /// Replaceable semantics: a strictly newer created_at replaces the
    /// author's list (ties keep the existing one, like the newest-wins
    /// floors in lib/platform/nostr/ndk_nostr_social_models.dart).
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

    /// Ingests a whole retrieval's events; anything that is not a
    /// kind-10002 relay list is ignored.
    pub fn ingest_all(&mut self, events: &[Event]) {
        for event in events {
            self.ingest(event);
        }
    }

    /// Remembers who the signed-in viewer follows. Dart's directory asks
    /// ndk for the contact list inside `discoveryRelayUrls`; Rust is told
    /// instead, by the bootstrap task that retrieved the kind-3. Holding
    /// it here is what lets a query built before the follow list landed
    /// still route by it.
    pub fn track_viewer_follows(&mut self, follows: Vec<PublicKey>) {
        self.viewer_follows = follows;
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
        ranked.sort_by(|left, right| counts[right].cmp(&counts[left]).then_with(|| left.cmp(right)));
        ranked.truncate(cap);
        ranked.into_iter().cloned().collect()
    }
}

/// Validated write relays of one relay list event, in first-seen
/// order. ndk keys r tags by cleaned url with the last marker winning,
/// then the app keeps entries whose marker allows writes.
fn write_urls(event: &Event) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    let mut writable: HashMap<String, bool> = HashMap::new();
    for tag in event.tags.iter() {
        let Some((url, write)) = write_declaration(tag.as_slice()) else {
            continue;
        };
        if !writable.contains_key(&url) {
            order.push(url.clone());
        }
        writable.insert(url, write);
    }
    order.retain(|url| writable[url]);
    order
}

/// An r tag as `(normalized url, declares write)`; `None` for non-r
/// tags and invalid urls. A missing or unknown marker means
/// read+write, so only an explicit `read` excludes the relay.
fn write_declaration(tag: &[String]) -> Option<(String, bool)> {
    if tag.len() < 2 || tag[0] != "r" {
        return None;
    }
    let url = normalize_relay_url(&tag[1])?;
    let write = tag.get(2).is_none_or(|marker| marker != "read");
    Some((url, write))
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
