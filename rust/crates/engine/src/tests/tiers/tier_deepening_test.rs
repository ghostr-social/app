use crate::tests::scheduling_support::{comfort, focus_at, hunger};
use crate::tiers::{classify, DemandSignals, Tier};
use crate::PostId;

#[test]
fn an_upcoming_tail_deepens_in_comfort() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(
        &PostId::new("b"),
        &focus,
        comfort(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, Some(Tier::T3Deepening));
}

#[test]
fn an_uncommitted_current_tail_deepens_in_comfort() {
    let focus = focus_at(&["a", "b"], 0, 500);

    let tier = classify(
        &PostId::new("a"),
        &focus,
        comfort(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, Some(Tier::T3Deepening));
}

#[test]
fn an_upcoming_tail_yields_nothing_in_hunger() {
    let focus = focus_at(&["a", "b"], 0, 0);

    let tier = classify(
        &PostId::new("b"),
        &focus,
        hunger(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, None);
}

#[test]
fn a_behind_tail_is_speculative_in_comfort() {
    let focus = focus_at(&["a", "b"], 1, 0);

    let tier = classify(
        &PostId::new("a"),
        &focus,
        comfort(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, Some(Tier::T4Speculative));
}

#[test]
fn a_behind_tail_yields_nothing_in_hunger() {
    let focus = focus_at(&["a", "b"], 1, 0);

    let tier = classify(
        &PostId::new("a"),
        &focus,
        hunger(true),
        DemandSignals::default(),
    );

    assert_eq!(tier, None);
}
