use crate::adaptive::{AdaptivePlayabilityPolicy, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn an_older_paid_transition_survives_a_one_transition_setback() {
    let mut input = snapshot(2, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 2;
    input.candidates[1].in_flight = vec![
        active(ByteRange::new(250_000, 500_000), 12_002),
        active(ByteRange::new(0, 250_000), 12_001),
    ];

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let transitions: Vec<_> = plan
        .retained
        .iter()
        .filter(|work| work.post == PostId::new("p1"))
        .collect();

    assert_eq!(transitions.len(), 1, "{plan:#?}");
    assert_eq!(
        transitions[0].request.requested_bytes(),
        ByteRange::new(0, 250_000)
    );
}

#[test]
fn emergency_preserves_one_paid_transition_when_the_hard_ceiling_allows_it() {
    let mut input = snapshot(2, 20_000_000, 1_000, 2);
    input.network.connection_capacity = 3;
    input.network.connection_ceiling = 3;
    input.candidates[1].in_flight = vec![active(ByteRange::new(0, 250_000), 12_001)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        plan.retained
            .iter()
            .any(|work| work.post == PostId::new("p1")),
        "{plan:#?}"
    );
}

fn active(bytes: ByteRange, committed_until_ms: u64) -> InFlightAction {
    let action = crate::ActionId::new(bytes.start.saturating_add(1));
    InFlightAction::range(
        action,
        bytes,
        "https://origin.example/media",
        committed_until_ms,
        true,
    )
}
