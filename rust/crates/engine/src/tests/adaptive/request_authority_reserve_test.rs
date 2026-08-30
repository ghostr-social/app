use super::warp_planner_test_assertions::set_source;
use crate::adaptive::{AdaptivePlayabilityPolicy, NextReserveEvidence, NextReserveInfeasibility};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn emergency_reserve_shares_slots_across_canonical_authority_aliases() {
    let mut input = snapshot(4, 20_000_000, 0, u16::MAX);
    input.network.connection_capacity = 1;
    set_source(&mut input, 0, "https://current.example/video");
    set_source(&mut input, 1, "https://EXAMPLE.com:443/one");
    set_source(&mut input, 2, "https://example.com/two");
    set_source(&mut input, 3, "https://other.example/three");

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let aliases = [PostId::new("p1"), PostId::new("p2")];
    let admitted = plan
        .allocations
        .iter()
        .filter(|work| aliases.contains(&work.post))
        .count();

    assert!(plan.ready_reserve.target >= 2, "{plan:#?}");
    assert_eq!(admitted, 2, "{plan:#?}");
}

#[test]
fn emergency_reserve_keeps_the_final_connection_for_current_playback() {
    let mut input = snapshot(3, 20_000_000, 0, u16::MAX);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 2;
    set_source(&mut input, 0, "https://current.example/video");
    set_source(&mut input, 1, "https://example.com/one");
    set_source(&mut input, 2, "https://example.com/two");

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let futures = [PostId::new("p1"), PostId::new("p2")];
    let admitted = plan
        .allocations
        .iter()
        .filter(|work| futures.contains(&work.post))
        .count();

    assert_eq!(admitted, 1, "{plan:#?}");
}

#[test]
fn legacy_reserve_does_not_schedule_an_invalid_request_origin() {
    let mut input = snapshot(2, 20_000_000, 20_000, 60);
    set_source(&mut input, 0, "https://current.example/video");
    set_source(&mut input, 1, "not a URL");

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(
        plan.next_reserve,
        NextReserveEvidence::Infeasible {
            post: PostId::new("p1"),
            reason: NextReserveInfeasibility::NoLiveOrigin,
        },
    );
}
