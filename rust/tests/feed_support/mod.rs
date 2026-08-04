#![allow(dead_code)]

//! Fixture builders for feed assembly, feed store, and profile store
//! tests: signed video events fed through the real parsing entry point
//! (`video_post_from_event`) — no relay IO anywhere.

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::discovery::event_parsing::{video_post_from_event, ParsedVideoPost};

/// A kind-1 note whose content links one direct mp4, the most common
/// video post shape on Nostr.
pub fn video_note(keys: &Keys, slug: &str, created_at: u64) -> Event {
    note_with_tags(keys, slug, Vec::new(), created_at)
}

/// A kind-1 video note additionally carrying one `t` hashtag tag.
pub fn hashtag_video_note(keys: &Keys, slug: &str, tag: &str, created_at: u64) -> Event {
    let tags = vec![vec!["t".to_owned(), tag.to_owned()]];
    note_with_tags(keys, slug, tags, created_at)
}

/// An addressable kind-34235 video event named by its `d` identifier.
pub fn addressable_video(keys: &Keys, identifier: &str, slug: &str, created_at: u64) -> Event {
    let tags = vec![vec!["d".to_owned(), identifier.to_owned()]];
    signed(keys, Kind::Custom(34235), &content(slug), tags, created_at)
}

/// A kind-0 profile metadata event with raw JSON content.
pub fn profile_event(keys: &Keys, content: &str, created_at: u64) -> Event {
    signed(keys, Kind::Metadata, content, Vec::new(), created_at)
}

/// Parses one fixture event through the production parsing entry point.
pub fn parsed(event: &Event) -> ParsedVideoPost {
    video_post_from_event(event).expect("fixture event parses")
}

/// Parses a whole page of fixture events in the given order.
pub fn parsed_posts(events: &[Event]) -> Vec<ParsedVideoPost> {
    events.iter().map(parsed).collect()
}

fn note_with_tags(
    keys: &Keys,
    slug: &str,
    tags: Vec<Vec<String>>,
    created_at: u64,
) -> Event {
    signed(keys, Kind::TextNote, &content(slug), tags, created_at)
}

fn content(slug: &str) -> String {
    format!("https://cdn.example/{slug}.mp4")
}

fn signed(
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
