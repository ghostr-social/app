use ghostr_engine::PostId;

const RETAINED_CANDIDATES: usize = 64;

#[derive(Clone)]
struct CandidateRank {
    post: PostId,
    discovered_at: u64,
}

#[derive(Default)]
pub(crate) struct CandidatePriority {
    candidates: Vec<CandidateRank>,
}

impl CandidatePriority {
    pub(super) fn rank(&mut self, post: PostId, discovered_at: u64) {
        if let Some(known) = self.candidates.iter_mut().find(|known| known.post == post) {
            known.discovered_at = discovered_at.max(known.discovered_at);
        } else {
            self.candidates.push(CandidateRank {
                post,
                discovered_at,
            });
        }
        self.candidates.sort_by(newest_first);
        self.candidates.truncate(RETAINED_CANDIDATES);
    }

    pub(super) fn ranked(&self) -> Vec<PostId> {
        self.candidates
            .iter()
            .map(|candidate| &candidate.post)
            .cloned()
            .collect()
    }
}

fn newest_first(left: &CandidateRank, right: &CandidateRank) -> core::cmp::Ordering {
    right
        .discovered_at
        .cmp(&left.discovered_at)
        .then_with(|| left.post.cmp(&right.post))
}
