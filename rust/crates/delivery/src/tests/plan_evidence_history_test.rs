use crate::delivery_events::command_channel;
use ghostr_engine::adaptive::{AllocationPlan, DiscoveryDemand};

#[test]
fn delivery_handle_observes_timestamped_plan_revisions_in_order() {
    let (handle, mut commands) = command_channel();

    commands.publish_plan(10, plan(DiscoveryDemand::Hold));
    commands.publish_plan(20, plan(DiscoveryDemand::Expand));

    let history = handle.plan_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].revision, 1);
    assert_eq!(history[0].observed_at_ms, 10);
    assert_eq!(history[0].plan.discovery_demand, DiscoveryDemand::Hold);
    assert_eq!(history[1].revision, 2);
    assert_eq!(history[1].observed_at_ms, 20);
    assert_eq!(history[1].plan.discovery_demand, DiscoveryDemand::Expand);
}

fn plan(discovery_demand: DiscoveryDemand) -> AllocationPlan {
    AllocationPlan {
        discovery_demand,
        ..AllocationPlan::default()
    }
}
