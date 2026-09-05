use crate::tests::adaptive_plan_assertions::allocated_posts;
use crate::tests::adaptive_plan_support::plan_with_packet_loss;

#[test]
fn measured_packet_loss_contracts_the_manager_plan_at_equal_throughput() {
    let clean = plan_with_packet_loss(0);
    let lossy = plan_with_packet_loss(6_000);

    assert!(allocated_posts(&clean).len() >= allocated_posts(&lossy).len());
}
