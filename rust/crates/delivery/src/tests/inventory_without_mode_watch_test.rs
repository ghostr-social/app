use crate::manager::state::DeliveryState;
use ghostr_engine::inventory_controller::{Mode, PresentRanges};
use ghostr_engine::{DataUsageLevel, EngineParams};

#[test]
fn inventory_observation_is_valid_before_a_mode_subscriber_is_attached() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);

    let inventory = state.observe_inventory(&PresentRanges::new(), &|_| 1);

    assert_eq!(inventory.mode, Mode::Comfort);
}
