use crate::focus::{FocusState, FocusUpdate};
use crate::tests::support::ids;
use crate::PostId;

fn update(window: &[&str], current_index: usize, watch_ms: u64) -> FocusUpdate {
    FocusUpdate {
        window: ids(window),
        current_index,
        watch_ms,
    }
}

#[test]
fn an_update_replaces_the_window_and_current_item() {
    let mut focus = FocusState::new();
    focus.update_focus(update(&["a", "b"], 0, 500));

    focus.update_focus(update(&["b", "c", "d"], 1, 250));

    assert_eq!(focus.window(), ids(&["b", "c", "d"]).as_slice());
    assert_eq!(focus.current(), Some(&PostId::new("c")));
    assert_eq!(focus.current_index(), 1);
    assert_eq!(focus.watch_ms(), 250);
}

#[test]
fn an_out_of_bounds_index_clamps_to_the_last_item() {
    let mut focus = FocusState::new();

    focus.update_focus(update(&["a", "b"], 9, 0));

    assert_eq!(focus.current_index(), 1);
    assert_eq!(focus.current(), Some(&PostId::new("b")));
}

#[test]
fn an_empty_window_resets_to_index_zero() {
    let mut focus = FocusState::new();
    focus.update_focus(update(&["a"], 0, 100));

    focus.update_focus(update(&[], 3, 0));

    assert_eq!(focus.current_index(), 0);
    assert_eq!(focus.current(), None);
}

#[test]
fn watch_time_at_the_threshold_marks_commitment() {
    let mut focus = FocusState::new();
    focus.update_focus(update(&["a"], 0, 2_999));
    assert!(!focus.is_committed(3_000));

    focus.update_focus(update(&["a"], 0, 3_000));

    assert!(focus.is_committed(3_000));
}
