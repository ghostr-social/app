use crate::qoe::WatchWindow;
use ghostr_engine::adaptive::PlayabilitySnapshot;
use ghostr_engine::watch_model::{WatchContext, WatchKey};
use ghostr_engine::PostId;

pub(super) struct CandidateInput {
    pub post: PostId,
    pub rank: usize,
    pub current: bool,
    pub context: WatchContext,
}

pub(super) fn candidates(
    snapshot: &PlayabilitySnapshot,
    window: &WatchWindow,
) -> Vec<CandidateInput> {
    let Some(candidates) = window.for_current(&snapshot.playback.current) else {
        return snapshot_candidates(snapshot);
    };
    candidates
        .iter()
        .enumerate()
        .map(|(rank, candidate)| CandidateInput {
            post: candidate.post.clone(),
            rank,
            current: rank == 0,
            context: candidate.context.clone(),
        })
        .collect()
}

fn snapshot_candidates(snapshot: &PlayabilitySnapshot) -> Vec<CandidateInput> {
    let candidates: Vec<_> = snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.feed_offset.value() >= 0)
        .take(WatchWindow::LIMIT)
        .enumerate()
        .map(|(rank, candidate)| CandidateInput {
            post: candidate.post.clone(),
            rank,
            current: candidate.feed_offset.value() == 0,
            context: WatchContext::new(
                WatchKey::digest(candidate.post.as_str()),
                (candidate.duration_ms > 0).then_some(candidate.duration_ms),
            ),
        })
        .collect();
    if candidates
        .first()
        .is_some_and(|candidate| candidate.current)
    {
        candidates
    } else {
        Vec::new()
    }
}
