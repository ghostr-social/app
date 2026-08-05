//! Signed fixture events for the feed FFI tests: pages reach the state
//! through the real parsing entry points — no relay IO anywhere.

use nostr_sdk::{Event, Keys, Kind};

pub(crate) use crate::api::tests::signed_event_fixture::{signed_event, SignedEventFixture};

/// A kind-1 note whose content links one direct mp4, the most common
/// video post shape on Nostr.
pub(crate) fn video_note(keys: &Keys, slug: &str, created_at: u64) -> Event {
    let content = format!("https://cdn.example/{slug}.mp4");
    signed_event(SignedEventFixture {
        keys,
        kind: Kind::TextNote,
        content: &content,
        tags: Vec::new(),
        created_at,
    })
}

/// A kind-0 profile metadata event with raw JSON content.
pub(crate) fn profile_event(keys: &Keys, content: &str, created_at: u64) -> Event {
    signed_event(SignedEventFixture {
        keys,
        kind: Kind::Metadata,
        content,
        tags: Vec::new(),
        created_at,
    })
}

/// A kind-10002 relay list declaring one write relay per given url.
pub(crate) fn relay_list_event(keys: &Keys, urls: &[&str], created_at: u64) -> Event {
    let tags = urls
        .iter()
        .map(|url| vec!["r".to_owned(), (*url).to_owned()])
        .collect();
    signed_event(SignedEventFixture {
        keys,
        kind: Kind::RelayList,
        content: "",
        tags,
        created_at,
    })
}
