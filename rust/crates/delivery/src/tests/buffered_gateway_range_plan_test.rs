use crate::tests::gateway_range_plan_fixture::buffered_demand_plan;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::ByteRange;

#[test]
fn buffered_gateway_read_ahead_keeps_ordinary_preemption_authority() {
    let demanded = ByteRange::new(9_000, 9_100);

    let work = buffered_demand_plan(demanded);
    let transfer = work
        .plan
        .allocations
        .iter()
        .find(|allocation| {
            let range = allocation.request.requested_bytes();
            range.start == demanded.start && range.end >= demanded.end
        })
        .expect("allocation starting at the demanded offset");

    assert!(!work.emergency, "buffered read-ahead is not a stall");
    assert_eq!(transfer.authority, PreemptionAuthority::Transition);
}
