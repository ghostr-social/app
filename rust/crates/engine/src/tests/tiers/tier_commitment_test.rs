use crate::inventory_controller::Mode;
use crate::tests::scheduling_support::{comfort, focus_at, hunger, state};
use crate::tiers::{classify, DemandSignals, Tier};
use crate::PostId;

fn committed() -> DemandSignals {
    DemandSignals {
        gateway_demand: false,
        buffer_below_emergency: false,
        viewer_committed: true,
    }
}

#[test]
fn a_committed_current_tail_is_finished_even_in_hunger() {
    let focus = focus_at(&["a", "b"], 0, 5_000);

    let tier = classify(&PostId::new("a"), &focus, hunger(true), committed());

    assert_eq!(tier, Some(Tier::T1CurrentTail));
}

#[test]
fn a_committed_current_tail_outranks_comfort_deepening() {
    let focus = focus_at(&["a", "b"], 0, 5_000);

    let tier = classify(&PostId::new("a"), &focus, comfort(true), committed());

    assert_eq!(tier, Some(Tier::T1CurrentTail));
}

#[test]
fn an_uncommitted_current_tail_yields_nothing_in_hunger() {
    let focus = focus_at(&["a", "b"], 0, 500);

    let tier = classify(
        &PostId::new("a"),
        &focus,
        hunger(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, None);
}

#[test]
fn commitment_on_a_non_current_post_does_not_make_its_tail_t1() {
    let focus = focus_at(&["a", "b"], 0, 5_000);

    let tier = classify(
        &PostId::new("b"),
        &focus,
        state(Mode::Comfort, true, true),
        committed(),
    );

    assert_eq!(tier, Some(Tier::T3Deepening));
}
