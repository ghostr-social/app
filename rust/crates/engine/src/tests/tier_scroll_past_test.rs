use crate::tests::scheduling_support::{comfort, focus_at, hunger};
use crate::tiers::{classify, DemandSignals, Tier};
use crate::PostId;

fn urgent() -> DemandSignals {
    DemandSignals {
        gateway_demand: true,
        buffer_below_emergency: true,
        viewer_committed: true,
    }
}

#[test]
fn a_post_outside_the_window_yields_no_head_work() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(
        &PostId::new("gone"),
        &focus,
        hunger(false),
        DemandSignals::default(),
    );

    assert_eq!(tier, None);
}

#[test]
fn a_post_outside_the_window_yields_no_tail_work_even_in_comfort() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(
        &PostId::new("gone"),
        &focus,
        comfort(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, None);
}

#[test]
fn even_urgent_demand_cannot_revive_a_departed_post() {
    let focus = focus_at(&["a", "b"], 0, 5_000);

    let tier = classify(&PostId::new("gone"), &focus, hunger(true), urgent());

    assert_eq!(tier, None);
}

#[test]
fn an_empty_window_classifies_nothing() {
    let focus = focus_at(&[], 0, 0);

    let tier = classify(&PostId::new("a"), &focus, hunger(false), urgent());

    assert_ne!(tier, Some(Tier::T0PlaybackEmergency));
    assert_eq!(tier, None);
}
