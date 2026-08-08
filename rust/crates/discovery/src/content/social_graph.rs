//! The session account's social graph consumed from relay events
//! (plan §5.2): kind-3 follow lists and NIP-51 kind-10000 mute lists.
//! Pure ingestion and lookup — lists are edited and published by Dart,
//! only consumed here. Mutes hide creators: a muted pubkey's posts are
//! filtered by author, never by hashtag or single event.
//! Private (encrypted) mute entries stay in Dart: decryption needs the
//! keys, and keys never cross the FFI.

use nostr_sdk::{Event, Kind, PublicKey, Timestamp};
use std::collections::HashSet;

/// Follows and mutes of one session pubkey; events signed by anyone
/// else are ignored.
#[derive(Debug)]
pub struct SocialGraph {
    session: PublicKey,
    follows: PubkeyList,
    mutes: PubkeyList,
}

/// One replaceable list of pubkeys: a strictly newer timestamp replaces
/// it; ties keep the accepted value.
#[derive(Debug, Default)]
struct PubkeyList {
    created_at: Option<Timestamp>,
    pubkeys: HashSet<PublicKey>,
}

impl SocialGraph {
    pub fn new(session_pubkey: PublicKey) -> Self {
        Self {
            session: session_pubkey,
            follows: PubkeyList::default(),
            mutes: PubkeyList::default(),
        }
    }

    /// Whether this graph already belongs to the given session pubkey;
    /// re-adopting the same viewer must not discard what was ingested.
    pub fn belongs_to(&self, viewer: &PublicKey) -> bool {
        self.session == *viewer
    }

    /// Ingests the session's kind-3 follow list or kind-10000 mute
    /// list; other kinds and other authors' lists are ignored. Reports
    /// whether the follow set was replaced.
    pub fn ingest(&mut self, event: &Event) -> bool {
        if event.pubkey != self.session {
            return false;
        }
        match event.kind {
            Kind::ContactList => self.follows.ingest(event),
            Kind::MuteList => {
                self.mutes.ingest(event);
                false
            }
            _ => false,
        }
    }

    /// Ingests a whole retrieval's events, reporting whether the follow
    /// set was replaced by any of them.
    pub fn ingest_all(&mut self, events: &[Event]) -> bool {
        events
            .iter()
            .fold(false, |changed, event| self.ingest(event) | changed)
    }

    /// The follow set in a stable order, for routing and for the relay
    /// lists the outbox bootstrap chases.
    pub fn follow_list(&self) -> Vec<PublicKey> {
        let mut follows: Vec<PublicKey> = self.follows.pubkeys.iter().copied().collect();
        follows.sort();
        follows
    }

    /// Whether posts by this creator are muted.
    pub fn is_muted(&self, author: &PublicKey) -> bool {
        self.mutes.pubkeys.contains(author)
    }
}

impl PubkeyList {
    /// Reports whether the list was replaced.
    fn ingest(&mut self, event: &Event) -> bool {
        if !self.accepts(event.created_at) {
            return false;
        }
        self.created_at = Some(event.created_at);
        self.pubkeys = listed_pubkeys(event);
        true
    }

    fn accepts(&self, created_at: Timestamp) -> bool {
        self.created_at.is_none_or(|current| created_at > current)
    }
}

fn listed_pubkeys(event: &Event) -> HashSet<PublicKey> {
    event
        .tags
        .iter()
        .filter_map(|tag| p_tag_pubkey(tag.as_slice()))
        .collect()
}

/// The pubkey of a well-formed public p tag; malformed hex is skipped.
fn p_tag_pubkey(tag: &[String]) -> Option<PublicKey> {
    if tag.len() < 2 || tag[0] != "p" {
        return None;
    }
    PublicKey::from_hex(&tag[1]).ok()
}
