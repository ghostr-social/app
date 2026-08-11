use crate::inventory_controller::{InventoryCounts, InventoryState, Mode};

#[test]
fn current_startability_follows_the_leading_inventory_count() {
    assert!(!state(0).current_startable());
    assert!(state(1).current_startable());
}

fn state(startable: usize) -> InventoryState {
    InventoryState {
        counts: InventoryCounts {
            considered: 2,
            startable,
            target: 2,
        },
        mode: Mode::Hunger,
    }
}
