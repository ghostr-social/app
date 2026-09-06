use crate::adaptive::{InFlightAction, PlayabilitySnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

pub(super) fn contended_state() -> PlayabilitySnapshot {
    let mut input = snapshot(2, 20_000_000, 0, 2);
    input.network.connection_capacity = 2;
    input.network.connection_ceiling = 2;
    input.network.per_authority_request_limit = 1;
    input.candidates[0].origins[0].source = "https://origin.example/current.mp4".into();
    input.candidates[1].origins[0].source = "https://origin.example/outgoing.mp4".into();
    input.candidates[1].in_flight.push(InFlightAction::range(
        ActionId::new(7),
        ByteRange::new(0, 250_000),
        "https://origin.example/outgoing.mp4",
        9_000,
        true,
    ));
    input
}
