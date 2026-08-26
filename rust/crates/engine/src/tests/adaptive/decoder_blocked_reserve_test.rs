use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;

#[test]
fn decoder_blocked_candidate_is_not_ready_or_reserved() {
    let mut input = snapshot(3, 20_000_000, 20_000, 8);
    let blocked = input.candidates[1].post.clone();
    input.candidates[1].direct_playback_blocked = true;
    input.candidates[1].present = input.candidates[1]
        .startup
        .as_ref()
        .expect("valid test fixture")
        .ranges()
        .to_vec();

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .ready_reserve
        .candidates
        .iter()
        .all(|candidate| candidate.post != blocked));
}
