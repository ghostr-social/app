use super::super::PlayabilitySnapshot;
use crate::PostId;
use std::collections::HashSet;

pub(super) fn selected(snapshot: &PlayabilitySnapshot) -> HashSet<PostId> {
    let candidates = super::super::reserve_model::candidates(snapshot);
    let count = super::super::reserve_model::target(snapshot, &candidates).count;
    let evidence = super::super::reserve_evidence::initial(&candidates);
    let mut selected: HashSet<_> = candidates
        .iter()
        .take(count)
        .map(|candidate| candidate.post.clone())
        .collect();
    selected.extend(
        candidates
            .iter()
            .zip(evidence)
            .filter(|(_, item)| super::super::reserve_evidence::is_protected_state(&item.state))
            .take(count)
            .map(|(candidate, _)| candidate.post.clone()),
    );
    selected
}
