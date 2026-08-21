use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, MediaLayout, RetrievalRequest, WholeBodyContract,
    WholeFetchReason,
};
use crate::tests::adaptive_support::snapshot;

#[test]
fn retained_commitment_preserves_a_whole_actions_request_kind() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    let candidate = &mut input.candidates[1];
    candidate.layout = MediaLayout::RequiresCompleteFile;
    let request = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact {
            expected_bytes: candidate.total_bytes.unwrap(),
        },
        reason: WholeFetchReason::PlannedCompletion,
    };
    candidate.in_flight.push(InFlightAction {
        action_id: crate::ActionId::new(1),
        request,
        effective_bytes: request.requested_bytes(),
        reserved_storage_bytes: request.reserved_network_bytes(),
        source: "https://origin.example/media".to_owned(),
        committed_until_ms: 12_000,
        identity_current: true,
        cancelling: false,
    });

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 1, "{plan:#?}");
    assert_eq!(plan.retained[0].request, request);
}
