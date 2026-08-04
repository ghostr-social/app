//! The session account's social graph consumed from relay events
//! (plan §5.2): kind-3 follow lists and NIP-51 kind-10000 mute lists.
//! Pure ingestion and lookup — lists are edited and published by Dart,
//! only consumed here. Mutes hide creators: a muted pubkey's posts are
//! filtered by author, never by hashtag or single event, mirroring
//! `_loadBlockedProfiles` (only `pubKeys` is read) in
//! lib/platform/nostr/ndk_nostr_social.dart and the blocked-creator
//! filter in lib/features/video_catalog/domain/video_feed_policy.dart.
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

/// One replaceable list of pubkeys: strictly newer created_at replaces
/// it, ties keep the existing list — mirrors the newest-wins floors in
/// lib/platform/nostr/ndk_nostr_social_models.dart.
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

    /// Ingests the session's kind-3 follow list or kind-10000 mute
    /// list; other kinds and other authors' lists are ignored.
    pub fn ingest(&mut self, event: &Event) {
        if event.pubkey != self.session {
            return;
        }
        match event.kind {
            Kind::ContactList => self.follows.ingest(event),
            Kind::MuteList => self.mutes.ingest(event),
            _ => {}
        }
    }

    /// Pubkeys the session follows (p tags of the newest kind-3).
    pub fn follows(&self) -> &HashSet<PublicKey> {
        &self.follows.pubkeys
    }

    /// Whether posts by this creator are muted.
    pub fn is_muted(&self, author: &PublicKey) -> bool {
        self.mutes.pubkeys.contains(author)
    }

    /// Whether the mute list hides this event: mutes filter by the
    /// event's author, matching `VideoFeedPolicy.select`.
    pub fn filters_event(&self, event: &Event) -> bool {
        self.is_muted(&event.pubkey)
    }
}

impl PubkeyList {
    fn ingest(&mut self, event: &Event) {
        if !self.accepts(event.created_at) {
            return;
        }
        self.created_at = Some(event.created_at);
        self.pubkeys = listed_pubkeys(event);
    }

    fn accepts(&self, created_at: Timestamp) -> bool {
        self.created_at
            .is_none_or(|current| created_at > current)
    }
}

fn listed_pubkeys(event: &Event) -> HashSet<PublicKey> {
    event
        .tags
        .iter()
        .filter_map(|tag| p_tag_pubkey(tag.as_slice()))
        .collect()
}

/// The pubkey of a well-formed public p tag; malformed hex is skipped
/// (ndk carries the raw string, but it can never resolve to a creator).
fn p_tag_pubkey(tag: &[String]) -> Option<PublicKey> {
    if tag.len() < 2 || tag[0] != "p" {
        return None;
    }
    PublicKey::from_hex(&tag[1]).ok()
}
