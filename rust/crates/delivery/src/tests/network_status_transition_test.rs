use crate::delivery_events::DeliveryNetworkStatus;
use crate::manager::state::network::NetworkStatusUpdate;
use crate::manager::state::DeliveryState;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::{DataUsageLevel, EngineParams};

#[test]
fn only_a_fresh_class_transition_requests_controller_reset() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);

    assert_eq!(
        state.apply_network_status(status(NetworkClass::Wifi, 5)),
        NetworkStatusUpdate::ClassChanged
    );
    assert_eq!(
        state.apply_network_status(status(NetworkClass::Cellular, 4)),
        NetworkStatusUpdate::Stale
    );
    assert_eq!(
        state.apply_network_status(status(NetworkClass::Cellular, 5)),
        NetworkStatusUpdate::Stale
    );
    assert_eq!(
        state.apply_network_status(status(NetworkClass::Wifi, 6)),
        NetworkStatusUpdate::Refreshed
    );
    assert_eq!(
        state.apply_network_status(status(NetworkClass::Cellular, 7)),
        NetworkStatusUpdate::ClassChanged
    );
}

fn status(network: NetworkClass, generation: u64) -> DeliveryNetworkStatus {
    DeliveryNetworkStatus::new(network, generation)
}
