use crate::manager::timeline::{TimelineCoordinator, TimelineEvidence};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
enum PublicationKind {
    Installed,
    TailNeeded,
}

struct TimelinePublication {
    evidence: TimelineEvidence,
    kind: PublicationKind,
}

#[derive(Default)]
pub(super) struct TimelinePublications {
    entries: HashMap<PostId, TimelinePublication>,
}

impl TimelinePublications {
    fn preserves(&self, post: &PostId, snapshot: &StoredMediaSnapshot, has_timeline: bool) -> bool {
        self.entries.get(post).is_some_and(|publication| {
            publication.matches_catalog(has_timeline)
                && publication.evidence.still_valid_in(snapshot)
        })
    }

    pub(super) fn retain(&mut self, posts: &HashSet<PostId>) {
        self.entries.retain(|post, _| posts.contains(post));
    }

    pub(super) fn remove(&mut self, post: &PostId) {
        self.entries.remove(post);
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }
}

impl TimelinePublication {
    fn matches_catalog(&self, has_timeline: bool) -> bool {
        matches!(self.kind, PublicationKind::Installed) == has_timeline
    }
}

impl TimelineCoordinator {
    pub(crate) fn preserves_publication(
        &self,
        post: &PostId,
        snapshot: &StoredMediaSnapshot,
        has_timeline: bool,
    ) -> bool {
        self.publications.preserves(post, snapshot, has_timeline)
    }

    pub(crate) fn publish_installed(&mut self, post: PostId, evidence: TimelineEvidence) {
        self.publish(post, evidence, PublicationKind::Installed);
    }

    pub(crate) fn publish_tail_needed(&mut self, post: PostId, evidence: TimelineEvidence) {
        self.publish(post, evidence, PublicationKind::TailNeeded);
    }

    pub(crate) fn forget_publication(&mut self, post: &PostId) {
        self.publications.remove(post);
    }

    fn publish(&mut self, post: PostId, evidence: TimelineEvidence, kind: PublicationKind) {
        self.publications
            .entries
            .insert(post, TimelinePublication { evidence, kind });
    }
}
