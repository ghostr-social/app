#![allow(dead_code)]

//! Fixture builders for discovery tests: signed replaceable list events
//! (NIP-65 relay lists, kind-3 follow lists, NIP-51 mute lists) fed
//! straight into the pure ingestion structs — no relay IO.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, PublicKey, Tag, Timestamp};

/// Signs a list event of `kind` carrying the given raw tag tuples.
pub fn list_event(keys: &Keys, kind: Kind, tags: Vec<Vec<String>>, created_at: u64) -> Event {
    let tags = tags
        .into_iter()
        .map(|parts| Tag::parse(parts).expect("fixture tag"));
    EventBuilder::new(kind, "")
        .tags(tags)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(keys)
        .expect("signed fixture event")
}

/// NIP-65 kind-10002 relay list event.
pub fn relay_list(keys: &Keys, tags: Vec<Vec<String>>, created_at: u64) -> Event {
    list_event(keys, Kind::RelayList, tags, created_at)
}

/// Kind-3 follow list event.
pub fn contact_list(keys: &Keys, tags: Vec<Vec<String>>, created_at: u64) -> Event {
    list_event(keys, Kind::ContactList, tags, created_at)
}

/// NIP-51 kind-10000 mute list event.
pub fn mute_list(keys: &Keys, tags: Vec<Vec<String>>, created_at: u64) -> Event {
    list_event(keys, Kind::MuteList, tags, created_at)
}

/// An `r` tag declaring a read+write relay.
pub fn r_tag(url: &str) -> Vec<String> {
    vec!["r".to_owned(), url.to_owned()]
}

/// An `r` tag with an explicit marker (`read`, `write`, or anything).
pub fn r_tag_marked(url: &str, marker: &str) -> Vec<String> {
    vec!["r".to_owned(), url.to_owned(), marker.to_owned()]
}

/// A `p` tag naming a pubkey by hex.
pub fn p_tag(pubkey: &PublicKey) -> Vec<String> {
    vec!["p".to_owned(), pubkey.to_hex()]
}

/// A relay list declaring plain write relays for `keys`.
pub fn write_relay_list(keys: &Keys, urls: &[&str], created_at: u64) -> Event {
    relay_list(
        keys,
        urls.iter().map(|url| r_tag(url)).collect(),
        created_at,
    )
}

/// A kind-1 note authored by `keys`, for mute filtering checks.
pub fn plain_note(keys: &Keys, created_at: u64) -> Event {
    list_event(keys, Kind::TextNote, Vec::new(), created_at)
}
