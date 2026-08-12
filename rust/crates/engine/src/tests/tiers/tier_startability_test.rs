use crate::inventory_controller::Mode;
use crate::tests::scheduling_support::{focus_at, state};
use crate::tiers::{classify, DemandSignals, Tier};
use crate::PostId;

// Current is "b": a is behind; c..e complete the four-post critical
// prefix (distances 1..=3); f onward are outside it.
const WINDOW: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i"];

fn head_tier_of(post: &str, mode: Mode) -> Option<Tier> {
    let focus = focus_at(WINDOW, 1, 0);
    let inventory = state(mode, false);
    classify(
        &PostId::new(post),
        &focus,
        inventory,
        DemandSignals::default(),
    )
}

#[test]
fn only_critical_upcoming_heads_are_eligible_while_the_target_is_unmet() {
    assert_eq!(head_tier_of("c", Mode::Hunger), Some(Tier::T2Startability));
    assert_eq!(head_tier_of("e", Mode::Hunger), Some(Tier::T2Startability));
    assert_eq!(head_tier_of("f", Mode::Hunger), None);
}

#[test]
fn a_critical_head_stays_startability_work_if_the_observation_is_inconsistent() {
    assert_eq!(head_tier_of("c", Mode::Comfort), Some(Tier::T2Startability));
}

#[test]
fn the_current_head_is_startability_work_even_when_the_target_is_met() {
    assert_eq!(head_tier_of("b", Mode::Comfort), Some(Tier::T2Startability));
}

#[test]
fn posts_beyond_the_protected_prefix_never_emit_work() {
    assert_eq!(head_tier_of("g", Mode::Hunger), None);
    assert_eq!(head_tier_of("i", Mode::Hunger), None);
    assert_eq!(head_tier_of("g", Mode::Comfort), None);

    let focus = focus_at(WINDOW, 1, 0);
    let complete = state(Mode::Comfort, true);
    assert_eq!(
        classify(
            &PostId::new("f"),
            &focus,
            complete,
            DemandSignals::default(),
        ),
        None
    );
}

#[test]
fn a_head_behind_the_viewer_is_speculative() {
    assert_eq!(head_tier_of("a", Mode::Hunger), Some(Tier::T4Speculative));
}
