//! NIP-18 wrappers admitted into Following feeds.

use crate::content::parsing::{video_post_from_event, ParsedVideoPost, MAX_REPOSTABLE_EVENT_BYTES};
use crate::content::repost_hint::valid_relay_hint;
use nostr_sdk::{Event, JsonUtil, Kind};

pub const REPOST_KIND: u16 = 6;
pub const GENERIC_REPOST_KIND: u16 = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepostProvenance {
    pub event_id: String,
    pub reposter_pubkey: String,
    pub kind: u16,
    pub reposted_at: u64,
    pub target: RepostTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepostTarget {
    SpecificEvent,
    Coordinate,
}

/// Parses direct video events and verified, fully embedded repost wrappers.
/// Empty-content wrappers require target resolution and are deferred.
pub fn feed_post_from_event(event: &Event) -> Option<ParsedVideoPost> {
    video_post_from_event(event).or_else(|| reposted_video_from_event(event))
}

pub fn reposted_video_from_event(wrapper: &Event) -> Option<ParsedVideoPost> {
    let wrapper_kind = verified_wrapper_kind(wrapper)?;
    let original = embedded_original(wrapper)?;
    if is_protected(&original) {
        return None;
    }
    resolved_repost(
        wrapper,
        &original,
        repost_target(wrapper, &original),
        wrapper_kind,
    )
}

pub(crate) fn resolved_repost(
    wrapper: &Event,
    original: &Event,
    target: RepostTarget,
    wrapper_kind: u16,
) -> Option<ParsedVideoPost> {
    original.verify().ok()?;
    if missing_specific_content(wrapper, original, target) {
        return None;
    }
    if !kind_pair_matches(wrapper_kind, original.kind)
        || !target_tags_match(wrapper, original, target)
    {
        return None;
    }
    let post = video_post_from_event(original)?;
    Some(with_provenance(post, wrapper, wrapper_kind, target))
}

fn missing_specific_content(wrapper: &Event, original: &Event, target: RepostTarget) -> bool {
    target == RepostTarget::SpecificEvent
        && wrapper.content.is_empty()
        && (original.kind.is_replaceable() || original.kind.is_addressable())
}

pub(crate) fn verified_wrapper_kind(wrapper: &Event) -> Option<u16> {
    let kind = wrapper.kind.as_u16();
    if ![REPOST_KIND, GENERIC_REPOST_KIND].contains(&kind) {
        return None;
    }
    wrapper.verify().ok()?;
    Some(kind)
}

fn embedded_original(wrapper: &Event) -> Option<Event> {
    if wrapper.content.is_empty() || wrapper.content.len() > MAX_REPOSTABLE_EVENT_BYTES {
        return None;
    }
    let original = Event::from_json(&wrapper.content).ok()?;
    original.verify().ok()?;
    Some(original)
}

fn with_provenance(
    mut post: ParsedVideoPost,
    wrapper: &Event,
    wrapper_kind: u16,
    target: RepostTarget,
) -> ParsedVideoPost {
    post.feed_sort_at = wrapper.created_at.as_u64();
    post.repost = Some(RepostProvenance {
        event_id: wrapper.id.to_hex(),
        reposter_pubkey: wrapper.pubkey.to_hex(),
        kind: wrapper_kind,
        reposted_at: wrapper.created_at.as_u64(),
        target,
    });
    post
}

fn repost_target(wrapper: &Event, original: &Event) -> RepostTarget {
    if original.kind.is_addressable()
        && original.tags.identifier().is_some()
        && tags_named(wrapper, "a").next().is_some()
    {
        RepostTarget::Coordinate
    } else {
        RepostTarget::SpecificEvent
    }
}

fn kind_pair_matches(wrapper_kind: u16, original_kind: Kind) -> bool {
    match wrapper_kind {
        REPOST_KIND => original_kind == Kind::TextNote,
        GENERIC_REPOST_KIND => original_kind != Kind::TextNote,
        _ => false,
    }
}

fn target_tags_match(wrapper: &Event, original: &Event, target: RepostTarget) -> bool {
    target_reference_matches(wrapper, original, target)
        && tags_match(wrapper, "p", &original.pubkey.to_hex())
        && kind_tag_matches(wrapper, original)
}

fn target_reference_matches(wrapper: &Event, original: &Event, target: RepostTarget) -> bool {
    match target {
        RepostTarget::SpecificEvent => {
            tags_named(wrapper, "a").next().is_none() && event_tag_matches(wrapper, original)
        }
        RepostTarget::Coordinate => address_tag_matches(wrapper, original),
    }
}

fn event_tag_matches(wrapper: &Event, original: &Event) -> bool {
    let expected = original.id.to_hex();
    let matching: Vec<_> = tags_named(wrapper, "e").collect();
    if matching.iter().any(|tag| tag.get(1) != Some(&expected)) {
        return false;
    }
    wrapper.kind.as_u16() != REPOST_KIND
        || (!matching.is_empty() && matching.iter().all(|tag| relay_hint(tag)))
}

fn relay_hint(tag: &[String]) -> bool {
    let Some(raw) = tag.get(2) else {
        return false;
    };
    valid_relay_hint(raw)
}

fn kind_tag_matches(wrapper: &Event, original: &Event) -> bool {
    tags_match(wrapper, "k", &original.kind.as_u16().to_string())
}

fn address_tag_matches(wrapper: &Event, original: &Event) -> bool {
    if !original.kind.is_addressable() {
        return false;
    }
    let Some(identifier) = original.tags.identifier() else {
        return false;
    };
    let coordinate = format!(
        "{}:{}:{}",
        original.kind.as_u16(),
        original.pubkey.to_hex(),
        identifier
    );
    tags_match(wrapper, "a", &coordinate) && tags_named(wrapper, "a").next().is_some()
}

fn tags_match(wrapper: &Event, name: &str, expected: &str) -> bool {
    let values: Vec<_> = tags_named(wrapper, name).collect();
    values
        .iter()
        .all(|tag| tag.get(1).map(String::as_str) == Some(expected))
}

fn tags_named<'a>(event: &'a Event, name: &'a str) -> impl Iterator<Item = &'a [String]> {
    event
        .tags
        .iter()
        .map(|tag| tag.as_slice())
        .filter(move |tag| tag.first().map(String::as_str) == Some(name))
}

fn is_protected(event: &Event) -> bool {
    event.tags.iter().any(|tag| tag.as_slice() == ["-"])
}
