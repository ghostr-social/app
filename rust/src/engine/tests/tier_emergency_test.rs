use crate::engine::tests::scheduling_support::{focus_at, hunger};
use crate::engine::tiers::{classify, DemandSignals, Tier};
use crate::engine::PostId;

fn demand(gateway: bool, low_buffer: bool) -> DemandSignals {
    DemandSignals {
        gateway_demand: gateway,
        buffer_below_emergency: low_buffer,
        viewer_committed: false,
    }
}

#[test]
fn gateway_demand_for_the_current_post_is_an_emergency() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(&PostId::new("a"), &focus, hunger(true), demand(true, false));

    assert_eq!(tier, Some(Tier::T0PlaybackEmergency));
}

#[test]
fn low_buffer_ahead_for_the_current_post_is_an_emergency() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(
        &PostId::new("a"),
        &focus,
        hunger(false),
        demand(false, true),
    );

    assert_eq!(tier, Some(Tier::T0PlaybackEmergency));
}

#[test]
fn demand_for_a_non_current_post_is_not_an_emergency() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(&PostId::new("b"), &focus, hunger(false), demand(true, true));

    assert_eq!(tier, Some(Tier::T2Startability));
}

#[test]
fn a_calm_current_post_is_not_an_emergency() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(
        &PostId::new("a"),
        &focus,
        hunger(false),
        demand(false, false),
    );

    assert_eq!(tier, Some(Tier::T2Startability));
}
