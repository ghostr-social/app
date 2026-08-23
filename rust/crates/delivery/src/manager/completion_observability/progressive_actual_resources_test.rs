use super::super::actual_resources;
use super::hedge_metric_fixture;
use ghostr_engine::adaptive::ResourceCost;

#[test]
fn decision_replay_receives_network_store_and_request_measurements() {
    let mut done = hedge_metric_fixture::done(7, true);
    done.received_bytes = 9;

    assert_eq!(actual_resources(&done), ResourceCost::new(9, 7, 0, 1));
}
