use crate::tests::gateway_range_plan_fixture::buffered_demand_plan;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::ByteRange;

#[test]
fn buffered_gateway_read_ahead_keeps_ordinary_preemption_authority() {
    let demanded = ByteRange::new(9_000, 9_100);

    let work = buffered_demand_plan(demanded);
    let transfer = work
        .transfers
        .iter()
        .find(|transfer| transfer.request.chunk.range == demanded)
        .expect("exact demanded range");

    assert!(!work.emergency, "buffered read-ahead is not a stall");
    assert_eq!(transfer.request.authority, PreemptionAuthority::Transition);
}
