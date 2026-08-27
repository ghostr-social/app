//! A real manager recovers Full GET only through one sparse probe and clean EOF.

mod delivery_fixture;

#[tokio::test]
async fn full_get_recovers_through_one_capped_probe_then_one_eof_trial() {
    delivery_fixture::full_recovery_scenario::run().await;
}
