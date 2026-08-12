use super::support::planned_transfer;
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::adaptive::PreemptionAuthority;
use std::collections::HashSet;

#[test]
fn capacity_reservation_does_not_cancel_a_policy_retained_range() {
    let retained = planned_transfer("retained", "same.example", PreemptionAuthority::Speculative);
    let urgent = planned_transfer("urgent", "same.example", PreemptionAuthority::Transition);
    let mut inflight = InFlightChunks::new();
    let attempt = inflight.next_attempt(retained.request.chunk.clone(), retained.identity.clone());
    let (handle, token) = cancel_pair();
    inflight.insert(
        &attempt,
        retained.request.clone(),
        "same.example".into(),
        retained.commitment_until_ms,
        handle,
    );

    inflight.reconcile_with_commitments(&[urgent], 1, &HashSet::from([retained.id()]));

    assert!(!token.is_cancelled());
    assert_eq!(inflight.len(), 1);
}
