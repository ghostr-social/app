//! Canonical post selection and ordering for one feed page: one newest
//! event per video coordinate, newest-first, with older unique rows
//! appended below the existing snapshot.

use std::collections::{HashMap, HashSet};

use crate::event_parsing::ParsedVideoPost;
use crate::feed_spec::FeedSpec;
use crate::social_graph::SocialGraph;

/// The rows one fetched page contributes to a feed: canonical, ordered,
/// and only what this feed's spec shows the viewer.
pub fn select_posts(
    spec: &FeedSpec,
    fetched: Vec<ParsedVideoPost>,
    graph: &SocialGraph,
) -> Vec<ParsedVideoPost> {
    canonical_posts(fetched)
        .into_iter()
        .filter(|post| spec.accepts(post, graph))
        .collect()
}

/// The same-video identity of a post: addressable video revisions share
/// `kind:pubkey:identifier`, everything else is its event id. The
/// identifier is compared exactly as published: padding names a distinct
/// coordinate.
pub fn post_coordinate(post: &ParsedVideoPost) -> String {
    if !(30_000..40_000).contains(&u32::from(post.kind)) {
        return post.event_id.clone();
    }
    match &post.published_identifier {
        Some(identifier) => {
            format!("{}:{}:{}", post.kind, post.author_pubkey, identifier)
        }
        None => post.event_id.clone(),
    }
}

/// One canonical post per coordinate — newest created_at wins, ties keep
/// the lexicographically smaller event id — ordered newest-first with
/// ascending-ID tiebreak.
pub fn canonical_posts(fetched: Vec<ParsedVideoPost>) -> Vec<ParsedVideoPost> {
    let mut selected: HashMap<String, ParsedVideoPost> = HashMap::new();
    for post in fetched {
        match selected.entry(post_coordinate(&post)) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(post);
            }
            std::collections::hash_map::Entry::Occupied(mut slot) => {
                if is_newer(&post, slot.get()) {
                    slot.insert(post);
                }
            }
        }
    }
    let mut posts: Vec<ParsedVideoPost> = selected.into_values().collect();
    posts.sort_by(newest_first);
    posts
}

/// Appends the incoming posts whose coordinate is not already present,
/// below the current list and in their given order; reports whether the
/// list changed.
pub fn append_new(current: &mut Vec<ParsedVideoPost>, incoming: Vec<ParsedVideoPost>) -> bool {
    let mut seen: HashSet<String> = current.iter().map(post_coordinate).collect();
    let before = current.len();
    for post in incoming {
        if seen.insert(post_coordinate(&post)) {
            current.push(post);
        }
    }
    current.len() != before
}

fn is_newer(incoming: &ParsedVideoPost, current: &ParsedVideoPost) -> bool {
    incoming.created_at > current.created_at
        || (incoming.created_at == current.created_at && incoming.event_id < current.event_id)
}

fn newest_first(left: &ParsedVideoPost, right: &ParsedVideoPost) -> std::cmp::Ordering {
    right
        .created_at
        .cmp(&left.created_at)
        .then_with(|| left.event_id.cmp(&right.event_id))
}
