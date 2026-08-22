use crate::tests::warp_hedge_plan_fixture::{mirror_plan_on_network, HedgeCase};
use ghostr_engine::origin_model::NetworkClass;

#[test]
fn hedge_tail_forecast_uses_the_current_network_class() {
    let wifi = mirror_plan_on_network(HedgeCase::BeforeTail, NetworkClass::Wifi);
    let cellular = mirror_plan_on_network(HedgeCase::BeforeTail, NetworkClass::Cellular);

    assert!(!cellular.hedge_tails.is_empty());
    assert_ne!(wifi.hedge_tails, cellular.hedge_tails);
}
