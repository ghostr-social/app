//! Materialized wrapper identities and the target events that support them.

use crate::content::repost_reference::{lookup_for_enrichment, RepostLookupTarget};
use crate::content::repost_resolution::feed_posts_from_events;
use nostr_sdk::{Event, EventId};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Default)]
pub(super) struct ResolvedRepostSupport {
    donors: BTreeMap<EventId, BTreeSet<EventId>>,
}

pub(super) fn exact_target_id(event: &Event) -> Option<EventId> {
    match lookup_for_enrichment(event)?.target {
        RepostLookupTarget::Event { id, .. } => Some(id),
        RepostLookupTarget::Coordinate { .. } => None,
    }
}

pub(super) fn verified_event_ids(events: &[Event]) -> HashSet<EventId> {
    events
        .iter()
        .filter(|event| event.verify().is_ok())
        .map(|event| event.id)
        .collect()
}

impl ResolvedRepostSupport {
    pub(super) fn new(events: &[Event], wrappers: &[Event]) -> Self {
        let posts = feed_posts_from_events(events);
        let mut support = Self::default();
        for post in &posts {
            let Some(repost) = post.repost.as_ref() else {
                continue;
            };
            let wrapper = EventId::from_hex(&repost.event_id).expect("verified wrapper id");
            let donor = EventId::from_hex(&post.event_id).expect("verified target id");
            support.donors.entry(wrapper).or_default().insert(donor);
        }
        support.add_coordinate_donors(&posts, wrappers);
        support
    }

    pub(super) fn materialized(&self, wrapper: &EventId) -> bool {
        self.donors.contains_key(wrapper)
    }

    pub(super) fn donors_for(&self, wrappers: &BTreeSet<EventId>) -> BTreeSet<EventId> {
        wrappers
            .iter()
            .filter_map(|wrapper| self.donors.get(wrapper))
            .flatten()
            .copied()
            .collect()
    }

    fn add_coordinate_donors(
        &mut self,
        posts: &[crate::content::parsing::ParsedVideoPost],
        wrappers: &[Event],
    ) {
        for wrapper in wrappers {
            let Some(RepostLookupTarget::Coordinate {
                author,
                kind,
                identifier,
            }) = lookup_for_enrichment(wrapper).map(|lookup| lookup.target)
            else {
                continue;
            };
            let coordinate = format!("{kind}:{author}:{identifier}");
            self.add_matching_posts(wrapper.id, &coordinate, posts);
        }
    }

    fn add_matching_posts(
        &mut self,
        wrapper: EventId,
        coordinate: &str,
        posts: &[crate::content::parsing::ParsedVideoPost],
    ) {
        let donors: BTreeSet<_> = posts
            .iter()
            .filter(|post| post.repost.is_none() && post.coordinate() == coordinate)
            .filter_map(|post| EventId::from_hex(&post.event_id).ok())
            .collect();
        if !donors.is_empty() {
            self.donors.insert(wrapper, donors);
        }
    }
}
