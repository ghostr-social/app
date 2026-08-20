use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction, RetrievalRequest};
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

#[test]
fn retained_commitment_returns_the_exact_launched_action_identity() {
    let action = ActionId::new(41);
    let bytes = ByteRange::new(0, 250_000);
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[1].in_flight.push(InFlightAction {
        action_id: action,
        request: RetrievalRequest::FetchRange {
            bytes,
            promotion: None,
        },
        effective_bytes: bytes,
        reserved_storage_bytes: bytes.len(),
        source: "origin".to_owned(),
        committed_until_ms: 12_000,
        identity_current: true,
        cancelling: false,
    });

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 1, "{plan:#?}");
    assert_eq!(plan.retained[0].action_id, action);
}
