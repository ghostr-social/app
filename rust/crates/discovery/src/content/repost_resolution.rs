//! Settled-batch resolution for reposts whose signed target is fetched separately.

use super::parsing::{video_post_from_event, ParsedVideoPost};
use super::repost_reference::{lookup_for_enrichment, RepostLookupTarget};
use super::reposts::{feed_post_from_event, resolved_repost, verified_wrapper_kind, RepostTarget};
use nostr_sdk::Event;
use std::collections::HashMap;

pub fn feed_posts_from_events(events: &[Event]) -> Vec<ParsedVideoPost> {
    let originals = OriginalIndex::new(events);
    events
        .iter()
        .filter_map(|event| {
            feed_post_from_event(event).or_else(|| originals.resolve_empty_repost(event))
        })
        .collect()
}

struct OriginalIndex<'a> {
    by_id: HashMap<String, &'a Event>,
    by_coordinate: HashMap<String, &'a Event>,
}

impl<'a> OriginalIndex<'a> {
    fn new(events: &'a [Event]) -> Self {
        let mut index = Self {
            by_id: HashMap::new(),
            by_coordinate: HashMap::new(),
        };
        for event in events {
            index.insert(event);
        }
        index
    }

    fn insert(&mut self, event: &'a Event) {
        if event.verify().is_err() {
            return;
        }
        let Some(post) = video_post_from_event(event) else {
            return;
        };
        self.by_id.insert(post.event_id.clone(), event);
        if event.kind.is_addressable() {
            self.insert_coordinate(post.coordinate(), event);
        }
    }

    fn insert_coordinate(&mut self, coordinate: String, event: &'a Event) {
        let current = self.by_coordinate.get(&coordinate).copied();
        if current.is_none_or(|stored| newer(event, stored)) {
            self.by_coordinate.insert(coordinate, event);
        }
    }

    fn resolve_empty_repost(&self, wrapper: &Event) -> Option<ParsedVideoPost> {
        if !wrapper.content.is_empty() {
            return None;
        }
        let lookup = lookup_for_enrichment(wrapper)?;
        let original = self.original(&lookup.target)?;
        let target = target_mode(&lookup.target);
        let wrapper_kind = verified_wrapper_kind(wrapper)?;
        resolved_repost(wrapper, original, target, wrapper_kind)
    }

    fn original(&self, target: &RepostLookupTarget) -> Option<&'a Event> {
        match target {
            RepostLookupTarget::Event { id, .. } => self.by_id.get(&id.to_hex()).copied(),
            RepostLookupTarget::Coordinate {
                author,
                kind,
                identifier,
            } => self
                .by_coordinate
                .get(&format!("{kind}:{author}:{identifier}"))
                .copied(),
        }
    }
}

fn target_mode(target: &RepostLookupTarget) -> RepostTarget {
    match target {
        RepostLookupTarget::Event { .. } => RepostTarget::SpecificEvent,
        RepostLookupTarget::Coordinate { .. } => RepostTarget::Coordinate,
    }
}

fn newer(incoming: &Event, current: &Event) -> bool {
    incoming.created_at > current.created_at
        || (incoming.created_at == current.created_at && incoming.id < current.id)
}
