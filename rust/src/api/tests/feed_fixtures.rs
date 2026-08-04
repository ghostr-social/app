//! Signed fixture events for the feed FFI tests: pages reach the state
//! through the real parsing entry points — no relay IO anywhere.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

/// A kind-1 note whose content links one direct mp4, the most common
/// video post shape on Nostr.
pub(crate) fn video_note(keys: &Keys, slug: &str, created_at: u64) -> Event {
    let content = format!("https://cdn.example/{slug}.mp4");
    signed_event(keys, Kind::TextNote, &content, Vec::new(), created_at)
}

/// A kind-0 profile metadata event with raw JSON content.
pub(crate) fn profile_event(keys: &Keys, content: &str, created_at: u64) -> Event {
    signed_event(keys, Kind::Metadata, content, Vec::new(), created_at)
}

/// A kind-10002 relay list declaring one write relay per given url.
pub(crate) fn relay_list_event(keys: &Keys, urls: &[&str], created_at: u64) -> Event {
    let tags = urls
        .iter()
        .map(|url| vec!["r".to_owned(), (*url).to_owned()])
        .collect();
    signed_event(keys, Kind::RelayList, "", tags, created_at)
}

pub(crate) fn signed_event(
    keys: &Keys,
    kind: Kind,
    content: &str,
    tags: Vec<Vec<String>>,
    created_at: u64,
) -> Event {
    let tags = tags
        .into_iter()
        .map(|parts| Tag::parse(parts).expect("fixture tag"));
    EventBuilder::new(kind, content)
        .tags(tags)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(keys)
        .expect("signed fixture event")
}
