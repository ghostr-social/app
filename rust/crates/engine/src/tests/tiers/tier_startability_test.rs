use crate::inventory_controller::Mode;
use crate::tests::scheduling_support::{focus_at, state};
use crate::tiers::{classify, DemandSignals, Tier};
use crate::PostId;

// Current is "b": a is behind; c..g are within the 6-wide startable
// window (distances 1..=5); h and i sit beyond it.
const WINDOW: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h", "i"];

fn head_tier_of(post: &str, target_met: bool) -> Option<Tier> {
    let focus = focus_at(WINDOW, 1, 0);
    let inventory = state(Mode::Hunger, target_met, false);
    classify(
        &PostId::new(post),
        &focus,
        inventory,
        DemandSignals::default(),
    )
}

#[test]
fn an_upcoming_head_is_startability_work_while_the_target_is_unmet() {
    assert_eq!(head_tier_of("c", false), Some(Tier::T2Startability));
    assert_eq!(head_tier_of("g", false), Some(Tier::T2Startability));
}

#[test]
fn an_upcoming_head_is_speculative_once_the_target_is_met() {
    assert_eq!(head_tier_of("c", true), Some(Tier::T4Speculative));
}

#[test]
fn the_current_head_is_startability_work_even_when_the_target_is_met() {
    assert_eq!(head_tier_of("b", true), Some(Tier::T2Startability));
}

#[test]
fn a_head_beyond_the_startable_window_is_speculative() {
    assert_eq!(head_tier_of("h", false), Some(Tier::T4Speculative));
    assert_eq!(head_tier_of("i", false), Some(Tier::T4Speculative));
}

#[test]
fn a_head_behind_the_viewer_is_speculative() {
    assert_eq!(head_tier_of("a", false), Some(Tier::T4Speculative));
}
