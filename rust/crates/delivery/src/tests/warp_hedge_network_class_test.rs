use crate::tests::warp_hedge_plan_fixture::{mirror_plan_on_network, HedgeCase};
use ghostr_engine::origin_model::NetworkClass;

#[test]
fn neither_network_class_schedules_a_tail_wake_for_unverified_range_races() {
    let wifi = mirror_plan_on_network(HedgeCase::BeforeTail, NetworkClass::Wifi);
    let cellular = mirror_plan_on_network(HedgeCase::BeforeTail, NetworkClass::Cellular);

    assert!(cellular.hedge_tails.is_empty());
    assert!(wifi.hedge_tails.is_empty());
}
