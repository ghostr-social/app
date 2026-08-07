use ghostr_engine::PostId;

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
    pub(crate) fn rank(&mut self, post: PostId, discovered_at: u64) {
        if let Some(known) = self.candidates.iter_mut().find(|known| known.post == post) {
            known.discovered_at = discovered_at.max(known.discovered_at);
            return;
        }
        self.candidates.push(CandidateRank {
            post,
            discovered_at,
        });
    }

    pub(crate) fn ranked(&self) -> Vec<PostId> {
        let mut candidates = self.candidates.clone();
        candidates.sort_by(|left, right| {
            right
                .discovered_at
                .cmp(&left.discovered_at)
                .then_with(|| left.post.cmp(&right.post))
        });
        candidates
            .into_iter()
            .map(|candidate| candidate.post)
            .collect()
    }
}
