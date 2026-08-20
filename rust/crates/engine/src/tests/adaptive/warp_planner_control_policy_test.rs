use crate::adaptive::{
    ActionKind, ContinuationDecision, ContinuationPolicy, HedgeInput, HedgePolicy, IdentityProof,
    SemanticCandidate, SemanticGuardrail, SemanticScore, TransportCensorReason,
};
use crate::{ActionId, ByteRange, PostId};

#[test]
fn continuation_hysteresis_has_distinct_continue_pause_and_abort_regions() {
    let policy = ContinuationPolicy::new(100, 80);
    assert_eq!(policy.decide(101), ContinuationDecision::Continue);
    assert_eq!(
        policy.decide(20),
        ContinuationDecision::FinishBlockThenReplan
    );
    assert_eq!(policy.decide(-81), ContinuationDecision::Abort);
}

#[test]
fn delayed_range_hedge_requires_tail_delay_value_and_verified_identity() {
    let input = HedgeInput::new(
        ActionId::new(7),
        ActionKind::FetchRange(ByteRange::new(0, 65_536)),
    )
    .with_timing(1_000, 900)
    .with_value(5_000, 1_000);
    assert!(!HedgePolicy::eligible(&input, IdentityProof::Unverified));
    assert!(HedgePolicy::eligible(
        &input,
        IdentityProof::VerifiedHash([9; 32])
    ));
}

#[test]
fn semantic_guardrail_uses_scores_when_present_and_labels_only_true_rescue() {
    let intended = SemanticCandidate::new(PostId::new("a"), SemanticScore::Known(1_000_000), false);
    let near = SemanticCandidate::new(PostId::new("b"), SemanticScore::Known(980_000), true);
    let far = SemanticCandidate::new(PostId::new("c"), SemanticScore::Known(500_000), true);
    let policy = SemanticGuardrail::new(2, 30_000);
    assert!(
        policy
            .admit(&intended, &[intended.clone(), near.clone(), far.clone()])
            .admissible
    );
    assert!(
        policy
            .admit(&near, &[intended.clone(), near.clone(), far.clone()])
            .admissible
    );
    assert!(
        !policy
            .admit(&far, &[intended.clone(), near, far.clone()])
            .admissible
    );
    let unavailable =
        SemanticCandidate::new(PostId::new("b"), SemanticScore::Known(980_000), false);
    let rescue = policy.admit(&far, &[intended, unavailable, far.clone()]);
    assert!(rescue.admissible && rescue.rescue);
    assert_eq!(
        rescue.censor,
        Some(TransportCensorReason::UnavoidableReadinessFailure)
    );
}
