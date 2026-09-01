#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/wait.rs"]
mod wait;

use crate::delivery_fixture::items::sized_item;
use crate::delivery_fixture::options::{serial_long_retry_options, DeliveryOptions};
pub use assertions::assert_no_selected_action;
use core::time::Duration;
use ghostr_delivery::delivery_events::{
    DeliveryFocus, FocusGeneration, FocusItem, FocusTransition,
};
pub use wait::{wait_for_attempts, wait_for_decision_successor, wait_for_failures, wait_for_focus};

pub fn options() -> DeliveryOptions {
    let mut options = serial_long_retry_options(2);
    options.tuning.retry.base = Duration::from_secs(300);
    options.tuning.retry.max = Duration::from_secs(300);
    options
}

pub fn window(healthy: &str, target: &str, unrelated: &str, generation: u64) -> DeliveryFocus {
    focus(
        vec![
            sized_item("current", healthy, 16, 1_000),
            sized_item("target", target, 16, 1_000),
            sized_item("unrelated", unrelated, 16, 1_000),
            sized_item("barrier", healthy, 16, 1_000),
        ],
        0,
        generation,
    )
}

pub fn focused(healthy: &str, target: &str, unrelated: &str, generation: u64) -> DeliveryFocus {
    let items = window(healthy, target, unrelated, generation).items;
    focus(items, 1, generation)
}

fn focus(items: Vec<FocusItem>, current_index: usize, generation: u64) -> DeliveryFocus {
    DeliveryFocus {
        items,
        previews: Vec::new(),
        current_index,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("focus generation"),
        transition: FocusTransition::UserNavigation,
        rescue: None,
    }
}
