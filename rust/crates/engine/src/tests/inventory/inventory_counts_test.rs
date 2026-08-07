use crate::focus::FocusUpdate;
use crate::inventory_controller::{InventoryController, InventoryCounts, Mode};
use crate::tests::inventory_support::scenario;
use crate::EngineParams;

fn controller() -> InventoryController {
    InventoryController::new(EngineParams::default())
}

#[test]
fn the_window_considers_at_most_startable_window_posts() {
    let s = scenario(10, 10);

    let state = controller().inventory_state(&s.catalog, &s.focus, &s.present);

    assert_eq!(
        state.counts,
        InventoryCounts {
            considered: 6,
            startable: 6,
            target: 4,
        }
    );
    assert_eq!(state.mode, Mode::Comfort);
}

#[test]
fn posts_behind_the_current_index_do_not_count() {
    let mut s = scenario(8, 3);
    s.focus.update_focus(FocusUpdate {
        window: s.posts.clone(),
        current_index: 3,
        watch_ms: 0,
    });

    let state = controller().inventory_state(&s.catalog, &s.focus, &s.present);

    assert_eq!(
        state.counts,
        InventoryCounts {
            considered: 5,
            startable: 0,
            target: 4,
        }
    );
}

#[test]
fn a_short_feed_clamps_the_target_to_what_exists() {
    let s = scenario(2, 2);

    let state = controller().inventory_state(&s.catalog, &s.focus, &s.present);

    assert_eq!(
        state.counts,
        InventoryCounts {
            considered: 2,
            startable: 2,
            target: 2,
        }
    );
    assert_eq!(state.mode, Mode::Comfort);
}

#[test]
fn an_empty_window_is_instant_comfort() {
    let s = scenario(0, 0);

    let state = controller().inventory_state(&s.catalog, &s.focus, &s.present);

    assert_eq!(
        state.counts,
        InventoryCounts {
            considered: 0,
            startable: 0,
            target: 0,
        }
    );
    assert_eq!(state.mode, Mode::Comfort);
}
