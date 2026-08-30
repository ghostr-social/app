use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_support::plan_existing;
use crate::tests::candidate_catalog_fixture::candidate;
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};
use std::collections::HashSet;

#[test]
fn bounded_provisional_future_posts_remain_retrieval_eligible() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    for candidate in [
        candidate("current", 3),
        candidate("next", 2),
        candidate("third", 1),
    ] {
        state.apply_candidate(candidate);
    }

    let work = plan_existing(state);
    let eligible: HashSet<_> = work
        .snapshot
        .expect("adaptive snapshot")
        .candidates
        .into_iter()
        .filter(|candidate| candidate.retrieval_eligible)
        .map(|candidate| candidate.post)
        .collect();

    assert_eq!(
        eligible,
        ["current", "next", "third"]
            .into_iter()
            .map(PostId::new)
            .collect()
    );
}
