use crate::engine::focus::{FocusState, FocusUpdate};
use crate::engine::tests::support::ids;
use crate::engine::PostId;

#[test]
fn distance_is_the_signed_offset_from_the_current_item() {
    let mut focus = FocusState::new();
    focus.update_focus(FocusUpdate {
        window: ids(&["a", "b", "c", "d"]),
        current_index: 1,
        watch_ms: 0,
    });

    let cases = [
        ("a", Some(-1)),
        ("b", Some(0)),
        ("c", Some(1)),
        ("d", Some(2)),
    ];
    for (id, expected) in cases {
        assert_eq!(focus.distance_of(&PostId::new(id)), expected, "{id}");
    }
}

#[test]
fn posts_outside_the_window_have_no_distance() {
    let mut focus = FocusState::new();
    focus.update_focus(FocusUpdate {
        window: ids(&["a", "b"]),
        current_index: 0,
        watch_ms: 0,
    });

    assert_eq!(focus.distance_of(&PostId::new("zz")), None);
}

#[test]
fn an_empty_focus_knows_no_distances_and_no_current_item() {
    let focus = FocusState::new();

    assert_eq!(focus.distance_of(&PostId::new("a")), None);
    assert_eq!(focus.current(), None);
    assert!(focus.window().is_empty());
}
