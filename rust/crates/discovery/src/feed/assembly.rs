//! Canonical post selection and ordering for one feed page: one newest
//! event per video coordinate, newest-first, with older unique rows
//! appended below the existing snapshot.

use std::collections::HashMap;

use crate::content::parsing::ParsedVideoPost;
use crate::content::reposts::RepostTarget;
/// One canonical post per coordinate — newest created_at wins, ties keep
/// the lexicographically smaller event id — ordered newest-first with
/// ascending-ID tiebreak.
pub fn canonical_posts(fetched: Vec<ParsedVideoPost>) -> Vec<ParsedVideoPost> {
    canonical_posts_from_axes(fetched.clone(), fetched)
}

pub(crate) fn canonical_posts_from_axes(
    contents: Vec<ParsedVideoPost>,
    occurrences: Vec<ParsedVideoPost>,
) -> Vec<ParsedVideoPost> {
    let mut selected: HashMap<String, CanonicalPost> = HashMap::new();
    for post in contents {
        selected
            .entry(post.coordinate())
            .or_default()
            .consider_content(post);
    }
    for post in occurrences {
        selected
            .entry(post.coordinate())
            .or_default()
            .consider_occurrence(post);
    }
    let mut posts: Vec<ParsedVideoPost> = selected
        .into_values()
        .filter_map(CanonicalPost::combined)
        .collect();
    posts.sort_by(newest_first);
    posts
}

fn is_newer(incoming: &ParsedVideoPost, current: &ParsedVideoPost) -> bool {
    if incoming.event_id == current.event_id {
        return delivery_changed(incoming, current);
    }
    incoming.created_at > current.created_at
        || (incoming.created_at == current.created_at && incoming.event_id < current.event_id)
}

fn delivery_changed(incoming: &ParsedVideoPost, current: &ParsedVideoPost) -> bool {
    incoming.meta != current.meta
        || incoming.metadata_evidence != current.metadata_evidence
        || incoming.renditions != current.renditions
}

fn is_newer_occurrence(incoming: &ParsedVideoPost, current: &ParsedVideoPost) -> bool {
    incoming.feed_sort_at > current.feed_sort_at
        || (incoming.feed_sort_at == current.feed_sort_at && occurrence_tiebreak(incoming, current))
}

fn occurrence_tiebreak(incoming: &ParsedVideoPost, current: &ParsedVideoPost) -> bool {
    match (incoming.repost.is_none(), current.repost.is_none()) {
        (true, false) => true,
        (false, true) => false,
        _ => incoming.activity_event_id() < current.activity_event_id(),
    }
}

fn newest_first(left: &ParsedVideoPost, right: &ParsedVideoPost) -> std::cmp::Ordering {
    right
        .feed_sort_at
        .cmp(&left.feed_sort_at)
        .then_with(|| left.activity_event_id().cmp(right.activity_event_id()))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CanonicalPost {
    content: Option<ParsedVideoPost>,
    occurrence: Option<ParsedVideoPost>,
}

impl CanonicalPost {
    pub(crate) fn consider_content(&mut self, post: ParsedVideoPost) {
        if self
            .content
            .as_ref()
            .is_none_or(|current| is_newer(&post, current))
        {
            self.content = Some(post);
        }
    }

    pub(crate) fn consider_occurrence(&mut self, post: ParsedVideoPost) {
        if self
            .occurrence
            .as_ref()
            .is_none_or(|current| is_newer_occurrence(&post, current))
        {
            self.occurrence = Some(post);
        }
    }

    fn combined(self) -> Option<ParsedVideoPost> {
        let occurrence = self.occurrence?;
        let specific = occurrence
            .repost
            .as_ref()
            .is_some_and(|repost| repost.target == RepostTarget::SpecificEvent);
        let mut content = if specific {
            occurrence.clone()
        } else {
            self.content?
        };
        content.feed_sort_at = occurrence.feed_sort_at;
        content.repost = occurrence.repost;
        Some(content)
    }

    pub(crate) fn projected(&self) -> Option<ParsedVideoPost> {
        self.clone().combined()
    }
}
