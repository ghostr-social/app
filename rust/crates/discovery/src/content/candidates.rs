//! Admission boundary for raw relay events.
//!
//! Events are parsed once, invalid media is rejected at the edge, and
//! addressable revisions share one stable candidate identity.

use crate::content::parsing::{video_post_from_event, ParsedVideoPost};
use nostr_sdk::Event;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub struct CandidateRegistry {
    parsed: HashMap<String, Option<ParsedVideoPost>>,
    canonical: HashMap<String, String>,
}

impl CandidateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inspect(&mut self, event: &Event) -> CandidateInspection {
        let event_id = event.id.to_hex();
        if let Some(post) = self.parsed.get(&event_id) {
            return repeated(post);
        }
        let post = video_post_from_event(event);
        self.parsed.insert(event_id.clone(), post.clone());
        let admission = self.canonical_admission(event_id, post.as_ref());
        CandidateInspection { post, admission }
    }

    pub fn inspect_all(&mut self, events: &[Event]) -> CandidateBatch {
        let mut posts = Vec::new();
        let mut admitted = Vec::new();
        for event in events {
            let inspected = self.inspect(event);
            posts.extend(inspected.post);
            admitted.extend(admitted_candidate(inspected.admission));
        }
        CandidateBatch { posts, admitted }
    }

    pub fn clear(&mut self) {
        self.parsed.clear();
        self.canonical.clear();
    }

    fn canonical_admission(
        &mut self,
        event_id: String,
        post: Option<&ParsedVideoPost>,
    ) -> CandidateAdmission {
        let Some(post) = post else {
            return CandidateAdmission::Rejected;
        };
        let coordinate = post.coordinate();
        let Some(current_id) = self.canonical.get(&coordinate) else {
            self.canonical.insert(coordinate, event_id);
            return CandidateAdmission::Accepted(VideoCandidate::new(post.clone()));
        };
        if !self.is_newer(post, current_id) {
            return CandidateAdmission::Duplicate;
        }
        self.canonical.insert(coordinate, event_id);
        CandidateAdmission::Replaced(VideoCandidate::new(post.clone()))
    }

    fn is_newer(&self, incoming: &ParsedVideoPost, current_id: &str) -> bool {
        let current = self
            .parsed
            .get(current_id)
            .and_then(Option::as_ref)
            .expect("canonical candidates always reference parsed posts");
        incoming.created_at > current.created_at
            || (incoming.created_at == current.created_at && incoming.event_id < current.event_id)
    }
}

fn repeated(post: &Option<ParsedVideoPost>) -> CandidateInspection {
    CandidateInspection {
        post: post.clone(),
        admission: if post.is_some() {
            CandidateAdmission::Duplicate
        } else {
            CandidateAdmission::Rejected
        },
    }
}

fn admitted_candidate(admission: CandidateAdmission) -> Option<VideoCandidate> {
    match admission {
        CandidateAdmission::Accepted(candidate) | CandidateAdmission::Replaced(candidate) => {
            Some(candidate)
        }
        CandidateAdmission::Duplicate | CandidateAdmission::Rejected => None,
    }
}
