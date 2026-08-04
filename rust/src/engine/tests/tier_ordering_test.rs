use crate::engine::tiers::Tier;

#[test]
fn tiers_order_from_most_to_least_urgent() {
    assert!(Tier::T0PlaybackEmergency < Tier::T1CurrentTail);
    assert!(Tier::T1CurrentTail < Tier::T2Startability);
    assert!(Tier::T2Startability < Tier::T3Deepening);
    assert!(Tier::T3Deepening < Tier::T4Speculative);
}
