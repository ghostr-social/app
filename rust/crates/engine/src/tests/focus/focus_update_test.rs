use crate::focus::{FocusState, FocusUpdate};
use crate::tests::support::ids;
use crate::PostId;

fn update(window: &[&str], current_index: usize) -> FocusUpdate {
    FocusUpdate {
        window: ids(window),
        current_index,
        watch_ms: 0,
    }
}

#[test]
fn an_update_replaces_the_window_and_current_item() {
    let mut focus = FocusState::new();
    focus.update_focus(update(&["a", "b"], 0));

    focus.update_focus(update(&["b", "c", "d"], 1));

    assert_eq!(focus.window(), ids(&["b", "c", "d"]).as_slice());
    assert_eq!(focus.current(), Some(&PostId::new("c")));
}

#[test]
fn an_out_of_bounds_index_clamps_to_the_last_item() {
    let mut focus = FocusState::new();

    focus.update_focus(update(&["a", "b"], 9));

    assert_eq!(focus.current(), Some(&PostId::new("b")));
}

#[test]
fn an_empty_window_resets_to_index_zero() {
    let mut focus = FocusState::new();
    focus.update_focus(update(&["a"], 0));

    focus.update_focus(update(&[], 3));

    assert_eq!(focus.current(), None);
}
