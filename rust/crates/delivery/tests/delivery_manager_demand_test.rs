//! A distant gateway demand jumps ahead of the ordinary current-video gap
//! without discarding useful bytes already in flight.

mod delivery_fixture;
#[path = "delivery_manager_demand_test/fixture.rs"]
mod fixture;
#[path = "delivery_manager_demand_test/plan.rs"]
mod plan;
#[path = "delivery_manager_demand_test/request.rs"]
mod request;
#[path = "delivery_manager_demand_test/store.rs"]
mod store;
#[path = "delivery_manager_demand_test/support.rs"]
mod support;

use ghostr_engine::ByteRange;
use support::DemandScenario;

const MEBIBYTE: u64 = 1024 * 1024;
const CHUNK: u64 = 256 * 1024;
const DEMANDED: ByteRange = ByteRange {
    start: 2 * CHUNK,
    end: 3 * CHUNK,
};

#[tokio::test]
async fn distant_gateway_demand_runs_before_the_ordinary_gap() {
    let mut scenario = DemandScenario::start().await;
    let (_lease, planned) = scenario.request_demand().await;
    scenario.assert_demand_plan(&planned);
    scenario.complete_in_priority_order().await;
}
