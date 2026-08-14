//! Verified target references carried by NIP-18 wrappers.

use super::parsing::MAX_REPOSTABLE_EVENT_BYTES;
use super::repost_hint::valid_relay_hint;
use super::reposts::{
    reposted_video_from_event, verified_wrapper_kind, GENERIC_REPOST_KIND, REPOST_KIND,
};
use nostr_sdk::{Event, EventId, Kind, PublicKey};
use std::collections::BTreeSet;

const EVENT_TAG: &str = "e";
const AUTHOR_TAG: &str = "p";
const KIND_TAG: &str = "k";
const ADDRESS_TAG: &str = "a";
const MAX_RELAY_HINTS: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepostLookup {
    pub target: RepostLookupTarget,
    pub relay_hints: Vec<String>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum RepostLookupTarget {
    Event {
        id: EventId,
        author: Option<PublicKey>,
        kind: Option<u16>,
    },
    Coordinate {
        author: PublicKey,
        kind: u16,
        identifier: String,
    },
}

pub(crate) fn lookup_for_enrichment(wrapper: &Event) -> Option<RepostLookup> {
    let lookup = reference_for_repost(wrapper)?;
    let needs_target = wrapper.content.is_empty()
        || matches!(lookup.target, RepostLookupTarget::Coordinate { .. });
    needs_target.then_some(lookup)
}

pub(crate) fn reference_for_repost(wrapper: &Event) -> Option<RepostLookup> {
    let wrapper_kind = verified_wrapper_kind(wrapper)?;
    if wrapper.content.len() > MAX_REPOSTABLE_EVENT_BYTES {
        return None;
    }
    if !wrapper.content.is_empty() && reposted_video_from_event(wrapper).is_none() {
        return None;
    }
    match uniform_tag_value(wrapper, ADDRESS_TAG)? {
        Some(address) => coordinate_lookup(wrapper, wrapper_kind, address),
        None => event_lookup(wrapper, wrapper_kind),
    }
}

fn coordinate_lookup(wrapper: &Event, wrapper_kind: u16, raw: String) -> Option<RepostLookup> {
    if wrapper_kind != GENERIC_REPOST_KIND {
        return None;
    }
    let (kind, author, identifier) = parse_coordinate(&raw)?;
    if kind == 1 || !Kind::from(kind).is_addressable() {
        return None;
    }
    optional_tags_match(wrapper, author, kind)?;
    Some(RepostLookup {
        target: RepostLookupTarget::Coordinate {
            author,
            kind,
            identifier,
        },
        relay_hints: coordinate_hints(wrapper),
    })
}

fn event_lookup(wrapper: &Event, wrapper_kind: u16) -> Option<RepostLookup> {
    let (id, hints) = event_reference(wrapper, wrapper_kind)?;
    let author = optional_author(wrapper)?;
    let tagged_kind = optional_kind(wrapper)?;
    if wrapper_kind == REPOST_KIND && tagged_kind.is_some_and(|value| value != 1) {
        return None;
    }
    let kind = (wrapper_kind == REPOST_KIND).then_some(1).or(tagged_kind);
    Some(RepostLookup {
        target: RepostLookupTarget::Event { id, author, kind },
        relay_hints: hints,
    })
}

fn parse_coordinate(raw: &str) -> Option<(u16, PublicKey, String)> {
    let mut parts = raw.splitn(3, ':');
    let kind = parts.next()?.parse().ok()?;
    let author = PublicKey::from_hex(parts.next()?).ok()?;
    let identifier = parts.next()?.to_owned();
    (!identifier.trim().is_empty()).then_some((kind, author, identifier))
}

fn optional_tags_match(wrapper: &Event, author: PublicKey, kind: u16) -> Option<()> {
    if optional_author(wrapper)?.is_some_and(|value| value != author) {
        return None;
    }
    if optional_kind(wrapper)?.is_some_and(|value| value != kind) {
        return None;
    }
    Some(())
}

fn optional_author(event: &Event) -> Option<Option<PublicKey>> {
    match uniform_tag_value(event, AUTHOR_TAG)? {
        Some(raw) => Some(Some(PublicKey::from_hex(&raw).ok()?)),
        None => Some(None),
    }
}

fn optional_kind(event: &Event) -> Option<Option<u16>> {
    match uniform_tag_value(event, KIND_TAG)? {
        Some(raw) => Some(Some(raw.parse().ok()?)),
        None => Some(None),
    }
}

fn event_reference(wrapper: &Event, wrapper_kind: u16) -> Option<(EventId, Vec<String>)> {
    let mut tags = tags_named(wrapper, EVENT_TAG);
    let tag = tags.next()?;
    if tags.next().is_some() {
        return None;
    }
    let id = EventId::from_hex(tag.get(1)?).ok()?;
    let hints = relay_hints(wrapper, EVENT_TAG);
    if wrapper_kind == REPOST_KIND && hints.len() != 1 {
        return None;
    }
    Some((id, hints))
}

fn uniform_tag_value(event: &Event, name: &str) -> Option<Option<String>> {
    let mut tags = tags_named(event, name);
    let Some(first) = tags.next() else {
        return Some(None);
    };
    let value = first.get(1)?;
    if tags.any(|tag| tag.get(1) != Some(value)) {
        return None;
    }
    Some(Some(value.clone()))
}

fn coordinate_hints(event: &Event) -> Vec<String> {
    let mut hints: BTreeSet<_> = relay_hints(event, ADDRESS_TAG).into_iter().collect();
    hints.extend(relay_hints(event, EVENT_TAG));
    hints.into_iter().take(MAX_RELAY_HINTS).collect()
}

fn relay_hints(event: &Event, tag_name: &str) -> Vec<String> {
    tags_named(event, tag_name)
        .filter_map(|tag| tag.get(2))
        .filter(|hint| valid_relay_hint(hint))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(MAX_RELAY_HINTS)
        .collect()
}

fn tags_named<'a>(event: &'a Event, name: &'a str) -> impl Iterator<Item = &'a [String]> {
    event
        .tags
        .iter()
        .map(|tag| tag.as_slice())
        .filter(move |tag| tag.first().map(String::as_str) == Some(name))
}
