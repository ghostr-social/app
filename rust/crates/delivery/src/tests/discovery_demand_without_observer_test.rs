use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::{DataUsageLevel, EngineParams};
use tokio::sync::watch;

#[test]
fn discovery_demand_is_safe_before_an_observer_is_installed() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.observe_discovery_demand(DiscoveryDemand::Expand);
    let (sender, receiver) = watch::channel(DiscoveryDemand::Hold);
    state.publish_discovery_demand(sender);

    state.observe_discovery_demand(DiscoveryDemand::Expand);

    assert_eq!(*receiver.borrow(), DiscoveryDemand::Expand);
}
