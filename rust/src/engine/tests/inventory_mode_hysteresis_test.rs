use crate::engine::inventory_controller::{next_mode, InventoryController, Mode};
use crate::engine::tests::inventory_support::scenario;
use crate::engine::EngineParams;

#[test]
fn a_new_controller_starts_hungry() {
    let controller = InventoryController::new(EngineParams::default());

    assert_eq!(controller.mode(), Mode::Hunger);
}

#[test]
fn the_mode_table_enters_comfort_at_target_and_leaves_below_target_minus_one() {
    let cases = [
        (Mode::Hunger, 4, 4, Mode::Comfort),
        (Mode::Hunger, 5, 4, Mode::Comfort),
        (Mode::Hunger, 3, 4, Mode::Hunger),
        (Mode::Comfort, 3, 4, Mode::Comfort),
        (Mode::Comfort, 2, 4, Mode::Hunger),
        (Mode::Hunger, 2, 4, Mode::Hunger),
        (Mode::Comfort, 0, 1, Mode::Comfort),
        (Mode::Hunger, 0, 0, Mode::Comfort),
    ];

    for (current, startable, target, expected) in cases {
        assert_eq!(
            next_mode(current, startable, target),
            expected,
            "{current:?} with {startable}/{target}"
        );
    }
}

#[test]
fn comfort_holds_at_one_below_target_and_breaks_below_that() {
    let mut controller = InventoryController::new(EngineParams::default());
    let mut observe = |startable: usize| {
        let s = scenario(6, startable);
        controller
            .inventory_state(&s.catalog, &s.focus, &s.present)
            .mode
    };

    assert_eq!(observe(4), Mode::Comfort);
    assert_eq!(observe(3), Mode::Comfort);
    assert_eq!(observe(2), Mode::Hunger);
    assert_eq!(observe(3), Mode::Hunger);
    assert_eq!(observe(4), Mode::Comfort);
}
