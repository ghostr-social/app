use crate::focus::{FocusState, FocusUpdate};
use crate::inventory_controller::Mode;
use crate::tests::support::ids;
use crate::tiers::PostInventory;

mod work_bench;

pub(super) use work_bench::WorkBench;

pub(super) fn focus_at(window: &[&str], current_index: usize, watch_ms: u64) -> FocusState {
    let mut focus = FocusState::new();
    focus.update_focus(FocusUpdate {
        window: ids(window),
        current_index,
        watch_ms,
    });
    focus
}

pub(super) fn state(mode: Mode, head_complete: bool) -> PostInventory {
    PostInventory {
        mode,
        startable_target: 4,
        head_complete,
    }
}

pub(super) fn hunger(head_complete: bool) -> PostInventory {
    state(Mode::Hunger, head_complete)
}

pub(super) fn comfort(head_complete: bool) -> PostInventory {
    state(Mode::Comfort, head_complete)
}
