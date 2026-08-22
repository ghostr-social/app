//! Admission boundary for raw relay events.
//!
//! Invalid media is rejected at the edge, addressable revisions share one
//! stable identity, and only compact revision metadata survives inspection.

use crate::content::blossom::BlossomServerStore;
use crate::content::parsing::ParsedVideoPost;
use crate::content::repost_resolution::feed_posts_from_events;
use crate::content::reposts::feed_post_from_event;
use crate::feed::assembly::CanonicalPost;
use nostr_sdk::Event;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};

const CANDIDATE_COORDINATE_RETENTION: usize = 10_000;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CandidateId(String);

impl CandidateId {
    pub fn for_post(post: &ParsedVideoPost) -> Self {
        let coordinate = post.coordinate();
        Self(format!("{:x}", Sha256::digest(coordinate.as_bytes())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoCandidate {
    pub id: CandidateId,
    pub post: ParsedVideoPost,
}

impl VideoCandidate {
    fn new(post: ParsedVideoPost) -> Self {
        Self {
            id: CandidateId::for_post(&post),
            post,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateAdmission {
    Accepted(VideoCandidate),
    Replaced(VideoCandidate),
    Duplicate,
    Rejected,
}

pub struct CandidateInspection {
    pub post: Option<ParsedVideoPost>,
    pub admission: CandidateAdmission,
}

pub struct CandidateBatch {
    pub posts: Vec<ParsedVideoPost>,
    pub admitted: Vec<VideoCandidate>,
}

#[derive(Debug)]
pub struct CandidateRegistry {
    canonical: HashMap<String, CanonicalPost>,
    coordinate_order: VecDeque<String>,
    blossom: BlossomServerStore,
    retention: usize,
}

impl Default for CandidateRegistry {
    fn default() -> Self {
        Self {
            canonical: HashMap::new(),
            coordinate_order: VecDeque::new(),
            blossom: BlossomServerStore::default(),
            retention: CANDIDATE_COORDINATE_RETENTION,
        }
    }
}

impl CandidateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inspect(&mut self, event: &Event) -> CandidateInspection {
        self.blossom.ingest(std::slice::from_ref(event));
        let mut post = feed_post_from_event(event);
        if let Some(post) = &mut post {
            self.blossom.enrich(post);
        }
        let admission = self.canonical_admission(post.as_ref());
        CandidateInspection { post, admission }
    }

    pub fn inspect_all(&mut self, events: &[Event]) -> CandidateBatch {
        self.blossom.ingest(events);
        let mut posts = Vec::new();
        let mut admitted = Vec::new();
        for mut post in feed_posts_from_events(events) {
            self.blossom.enrich(&mut post);
            let admission = self.canonical_admission(Some(&post));
            posts.push(post);
            admitted.extend(admitted_candidate(admission));
        }
        CandidateBatch { posts, admitted }
    }

    pub fn clear(&mut self) {
        self.canonical.clear();
        self.coordinate_order.clear();
        self.blossom.clear();
    }

    fn canonical_admission(&mut self, post: Option<&ParsedVideoPost>) -> CandidateAdmission {
        let Some(post) = post else {
            return CandidateAdmission::Rejected;
        };
        let coordinate = post.coordinate();
        let replacing = self.canonical.contains_key(&coordinate);
        if !replacing {
            self.admit_coordinate(coordinate.clone());
        }
        let projection = self.canonical.entry(coordinate).or_default();
        let before = projection.projected();
        let retained = retained_post(post);
        projection.consider_content(retained.clone());
        projection.consider_occurrence(retained);
        let after = projection
            .projected()
            .expect("a considered post always projects a candidate");
        if before
            .as_ref()
            .is_some_and(|before| same_candidate(before, &after))
        {
            return CandidateAdmission::Duplicate;
        }
        candidate_admission(VideoCandidate::new(after), replacing)
    }

    fn admit_coordinate(&mut self, coordinate: String) {
        if self.canonical.len() >= self.retention {
            let evicted = self
                .coordinate_order
                .pop_front()
                .expect("canonical coordinates share one admission order");
            self.canonical.remove(&evicted);
        }
        self.coordinate_order.push_back(coordinate);
    }

    #[cfg(test)]
    pub(crate) fn with_retention(retention: usize) -> Self {
        Self {
            retention: retention.max(1),
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn retained_coordinates(&self) -> usize {
        self.canonical.len()
    }
}

fn candidate_admission(candidate: VideoCandidate, replacing: bool) -> CandidateAdmission {
    if replacing {
        CandidateAdmission::Replaced(candidate)
    } else {
        CandidateAdmission::Accepted(candidate)
    }
}

fn retained_post(post: &ParsedVideoPost) -> ParsedVideoPost {
    let mut retained = post.clone();
    retained.signed_event_json = None;
    retained
}

fn same_candidate(left: &ParsedVideoPost, right: &ParsedVideoPost) -> bool {
    left.event_id == right.event_id
        && left.feed_sort_at == right.feed_sort_at
        && left.meta == right.meta
        && left.metadata_evidence == right.metadata_evidence
        && left.renditions == right.renditions
}

fn admitted_candidate(admission: CandidateAdmission) -> Option<VideoCandidate> {
    match admission {
        CandidateAdmission::Accepted(candidate) | CandidateAdmission::Replaced(candidate) => {
            Some(candidate)
        }
        CandidateAdmission::Duplicate | CandidateAdmission::Rejected => None,
    }
}
