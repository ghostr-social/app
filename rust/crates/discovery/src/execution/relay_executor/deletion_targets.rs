//! Verified deletion targets and the repost wrappers that depend on them.

use super::deletion_hints::DeletionHints;
use crate::content::parsing::ParsedVideoPost;
use crate::content::repost_resolution::feed_posts_from_events;
use crate::content::reposts::RepostTarget;
use nostr_sdk::{Event, EventId, PublicKey};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct DeletionTarget {
    pub(super) author: PublicKey,
    pub(super) value: String,
    pub(super) hints: Vec<String>,
    pub(super) dependents: BTreeSet<EventId>,
    pub(super) rank: usize,
}

#[derive(Default)]
pub(super) struct DeletionTargets {
    pub(super) events: Vec<DeletionTarget>,
    pub(super) addresses: Vec<DeletionTarget>,
}

#[derive(Clone)]
struct WrapperEvidence {
    dependents: BTreeSet<EventId>,
    rank: usize,
}

pub(super) fn deletion_targets(events: &[Event]) -> DeletionTargets {
    let posts = feed_posts_from_events(events);
    let index = DependencyIndex::new(&posts);
    let hints = DeletionHints::from_events(events);
    let mut targets = DeletionTargets::default();
    for post in &posts {
        add_post_targets(post, &index, &hints, &mut targets);
    }
    targets
}

fn add_post_targets(
    post: &ParsedVideoPost,
    index: &DependencyIndex,
    hints: &DeletionHints,
    targets: &mut DeletionTargets,
) {
    let dependents = index.dependencies(post);
    if dependents.is_empty() {
        return;
    }
    let evidence = WrapperEvidence {
        rank: index.rank(&dependents),
        dependents,
    };
    add_wrapper_target(post, &evidence, targets);
    add_original_targets(post, evidence, hints, targets);
}

fn add_original_targets(
    post: &ParsedVideoPost,
    evidence: WrapperEvidence,
    hints: &DeletionHints,
    targets: &mut DeletionTargets,
) {
    let author = PublicKey::from_hex(&post.author_pubkey).expect("parsed post author");
    targets.events.push(target(
        author,
        post.event_id.clone(),
        hints.for_post(post),
        evidence.clone(),
    ));
    if post.published_identifier.is_some() {
        let coordinate = post.coordinate();
        targets.addresses.push(target(
            author,
            coordinate.clone(),
            hints.for_coordinate(&coordinate),
            evidence,
        ));
    }
}

fn add_wrapper_target(
    post: &ParsedVideoPost,
    evidence: &WrapperEvidence,
    targets: &mut DeletionTargets,
) {
    let Some(repost) = post.repost.as_ref() else {
        return;
    };
    let author = PublicKey::from_hex(&repost.reposter_pubkey).expect("verified reposter");
    targets.events.push(target(
        author,
        repost.event_id.clone(),
        Vec::new(),
        evidence.clone(),
    ));
}

fn target(
    author: PublicKey,
    value: String,
    hints: Vec<String>,
    evidence: WrapperEvidence,
) -> DeletionTarget {
    DeletionTarget {
        author,
        value,
        hints,
        dependents: evidence.dependents,
        rank: evidence.rank,
    }
}

struct DependencyIndex {
    coordinates: BTreeMap<String, BTreeSet<EventId>>,
    ranks: BTreeMap<EventId, usize>,
}

impl DependencyIndex {
    fn new(posts: &[ParsedVideoPost]) -> Self {
        let mut index = Self {
            coordinates: BTreeMap::new(),
            ranks: BTreeMap::new(),
        };
        for (rank, post) in posts.iter().enumerate() {
            index.insert(post, rank);
        }
        index
    }

    fn insert(&mut self, post: &ParsedVideoPost, rank: usize) {
        let Some(repost) = post.repost.as_ref() else {
            return;
        };
        let id = EventId::from_hex(&repost.event_id).expect("verified wrapper id");
        self.ranks.entry(id).or_insert(rank);
        if repost.target == RepostTarget::Coordinate {
            self.coordinates
                .entry(post.coordinate())
                .or_default()
                .insert(id);
        }
    }

    fn dependencies(&self, post: &ParsedVideoPost) -> BTreeSet<EventId> {
        match post.repost.as_ref() {
            Some(repost) => EventId::from_hex(&repost.event_id).into_iter().collect(),
            None => self
                .coordinates
                .get(&post.coordinate())
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn rank(&self, dependents: &BTreeSet<EventId>) -> usize {
        dependents
            .iter()
            .filter_map(|id| self.ranks.get(id))
            .copied()
            .min()
            .unwrap_or(usize::MAX)
    }
}
