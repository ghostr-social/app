//! Verified NIP-18 routes attached only to their referenced originals.

use super::target_hints::{normalized_hints, MAX_HINTS_PER_TARGET};
use crate::content::parsing::ParsedVideoPost;
use crate::content::repost_reference::{reference_for_repost, RepostLookupTarget};
use nostr_sdk::Event;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct DeletionHints {
    events: BTreeMap<String, BTreeSet<String>>,
    coordinates: BTreeMap<String, BTreeSet<String>>,
}

impl DeletionHints {
    pub(super) fn from_events(events: &[Event]) -> Self {
        let mut hints = Self::default();
        for lookup in events.iter().filter_map(reference_for_repost) {
            hints.insert(lookup.target, normalized_hints(lookup.relay_hints));
        }
        hints
    }

    pub(super) fn for_post(&self, post: &ParsedVideoPost) -> Vec<String> {
        let mut hints = self.events.get(&post.event_id).cloned().unwrap_or_default();
        if post.published_identifier.is_some() {
            hints.extend(
                self.coordinates
                    .get(&post.coordinate())
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        }
        hints.into_iter().collect()
    }

    pub(super) fn for_coordinate(&self, coordinate: &str) -> Vec<String> {
        self.coordinates
            .get(coordinate)
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }

    fn insert(&mut self, target: RepostLookupTarget, hints: Vec<String>) {
        let (index, key) = match target {
            RepostLookupTarget::Event { id, .. } => (&mut self.events, id.to_hex()),
            RepostLookupTarget::Coordinate {
                author,
                kind,
                identifier,
            } => (
                &mut self.coordinates,
                format!("{kind}:{}:{identifier}", author.to_hex()),
            ),
        };
        let retained = index.entry(key).or_default();
        retained.extend(hints);
        while retained.len() > MAX_HINTS_PER_TARGET {
            retained.pop_last();
        }
    }
}
