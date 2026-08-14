use super::{FeedId, FeedStore, OpenFeed, QUERY_POST_RETENTION};
use crate::content::deletions::DeletionClaim;
use crate::content::parsing::ParsedVideoPost;
use crate::content::social_graph::SocialGraph;
use crate::feed::assembly::canonical_posts_from_axes;
use std::collections::{HashMap, HashSet};

const OCCURRENCES_PER_COORDINATE: usize = 4;

impl FeedStore {
    pub fn ingest_deletions(
        &mut self,
        feed: FeedId,
        claims: Vec<DeletionClaim>,
        graph: &SocialGraph,
    ) -> bool {
        if claims.is_empty() {
            return false;
        }
        let Some(open) = self.feeds.get_mut(&feed) else {
            return false;
        };
        if !open.deletions.ingest(claims) {
            return false;
        }
        let before = open.posts.clone();
        open.reproject(graph);
        let changed = open.posts != before;
        if changed {
            open.notify();
        }
        changed
    }
}

impl OpenFeed {
    pub(super) fn add_occurrences(
        &mut self,
        fetched: Vec<ParsedVideoPost>,
        graph: &SocialGraph,
    ) -> bool {
        let before = self.posts.clone();
        self.occurrences.extend(fetched);
        self.compact_occurrences();
        self.reproject(graph);
        let changed = self.posts != before;
        if changed {
            self.notify();
        }
        changed
    }

    pub(super) fn compact_occurrences(&mut self) {
        self.occurrences.sort_by(|left, right| {
            right
                .feed_sort_at
                .cmp(&left.feed_sort_at)
                .then_with(|| left.activity_event_id().cmp(right.activity_event_id()))
        });
        let mut seen = HashSet::new();
        self.occurrences
            .retain(|post| seen.insert(post.activity_event_id().to_owned()));
        retain_fair_occurrences(&mut self.occurrences);
        self.occurrences.truncate(QUERY_POST_RETENTION);
    }

    pub(super) fn reproject(&mut self, graph: &SocialGraph) {
        self.deletions.reanchor(&self.occurrences);
        let contents = self
            .occurrences
            .iter()
            .filter(|post| self.spec.accepts_content(post, graph))
            .filter(|post| !self.deletes_content(post))
            .cloned()
            .collect();
        let occurrences = self
            .occurrences
            .iter()
            .filter(|post| self.spec.accepts(post, graph))
            .filter(|post| !self.deletes_occurrence(post))
            .cloned()
            .collect();
        self.posts = canonical_posts_from_axes(contents, occurrences);
        self.trim();
    }

    fn deletes_content(&self, post: &ParsedVideoPost) -> bool {
        self.deletions.deletes_content(post)
    }

    fn deletes_occurrence(&self, post: &ParsedVideoPost) -> bool {
        self.deletions.deletes_occurrence(post)
    }
}

fn retain_fair_occurrences(posts: &mut Vec<ParsedVideoPost>) {
    let mut retention = FairOccurrenceRetention::new(posts);
    posts.retain(|post| retention.keeps(post));
}

struct FairOccurrenceRetention {
    latest: HashMap<String, String>,
    counts: HashMap<String, usize>,
    latest_kept: HashSet<String>,
    direct_kept: HashSet<String>,
}

impl FairOccurrenceRetention {
    fn new(posts: &[ParsedVideoPost]) -> Self {
        Self {
            latest: latest_content_revisions(posts),
            counts: HashMap::new(),
            latest_kept: HashSet::new(),
            direct_kept: HashSet::new(),
        }
    }

    fn keeps(&mut self, post: &ParsedVideoPost) -> bool {
        let coordinate = post.coordinate();
        let is_latest = self.latest.get(&coordinate) == Some(&post.event_id);
        if !self.has_capacity(&coordinate) && !self.is_required(post, &coordinate, is_latest) {
            return false;
        }
        self.record(post, coordinate, is_latest);
        true
    }

    fn has_capacity(&self, coordinate: &str) -> bool {
        self.counts.get(coordinate).copied().unwrap_or(0) < OCCURRENCES_PER_COORDINATE
    }

    fn is_required(&self, post: &ParsedVideoPost, coordinate: &str, is_latest: bool) -> bool {
        let needs_latest = is_latest && !self.latest_kept.contains(coordinate);
        let needs_direct = post.repost.is_none() && !self.direct_kept.contains(coordinate);
        needs_latest || needs_direct
    }

    fn record(&mut self, post: &ParsedVideoPost, coordinate: String, is_latest: bool) {
        *self.counts.entry(coordinate.clone()).or_default() += 1;
        if is_latest {
            self.latest_kept.insert(coordinate.clone());
        }
        if post.repost.is_none() {
            self.direct_kept.insert(coordinate);
        }
    }
}

fn latest_content_revisions(posts: &[ParsedVideoPost]) -> HashMap<String, String> {
    let mut latest = HashMap::<String, (u64, String)>::new();
    for post in posts {
        let coordinate = post.coordinate();
        let revision = (post.created_at, post.event_id.clone());
        let current = latest.entry(coordinate).or_insert_with(|| revision.clone());
        if revision.0 > current.0 || (revision.0 == current.0 && revision.1 < current.1) {
            *current = revision;
        }
    }
    latest
        .into_iter()
        .map(|(coordinate, (_, event_id))| (coordinate, event_id))
        .collect()
}
