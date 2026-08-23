use ghostr_engine::adaptive::BOOTSTRAP_DIRECT_FETCH_BYTES;

#[test]
fn degraded_network_preserves_the_bootstrap_whole_body_allowance() {
    let (burst, rate) = super::limits::network_budget(80_000);

    assert_eq!(rate, 10_000);
    assert_eq!(burst, BOOTSTRAP_DIRECT_FETCH_BYTES);
}
