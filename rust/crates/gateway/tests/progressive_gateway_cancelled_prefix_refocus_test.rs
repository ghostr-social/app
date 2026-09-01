mod gateway_fixture;
#[path = "progressive_gateway_cancelled_prefix_refocus_test/support/mod.rs"]
mod support;

use support::CancelledPrefixScenario;

#[tokio::test]
async fn recreated_gateway_demand_refills_a_zero_byte_cancelled_prefix() {
    let mut scenario = CancelledPrefixScenario::start().await;
    scenario.cancel_speculative_prefix().await;
    scenario.advance_before_target().await;
    scenario.expect_first_active_demand_refill().await;
}
