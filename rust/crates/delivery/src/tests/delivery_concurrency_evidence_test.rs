use crate::manager::concurrency::{capacity_evidence, connection_ceiling, network_profile_setback};
use crate::manager::traffic::OverallTrafficWindow;
use ghostr_engine::concurrency::{ConcurrencyOccupancy, NetworkSetback};
use std::time::Duration;

#[test]
fn traffic_windows_preserve_aggregate_rate_occupancy_and_latency() {
    let window = OverallTrafficWindow::new(
        1_000_000,
        Duration::from_millis(500),
        3,
        10,
        Some(Duration::from_millis(120)),
    );

    let evidence = capacity_evidence(window, true, Duration::from_secs(1), 3);

    assert_eq!(evidence.aggregate_bytes_per_second, 2_000_000);
    assert_eq!(evidence.occupancy, ConcurrencyOccupancy::new(3, 3));
    assert!(evidence.saturated);
    assert_eq!(evidence.ttfb, Duration::from_millis(120));
}

#[test]
fn observed_packet_loss_is_an_immediate_concurrency_setback() {
    assert_eq!(network_profile_setback(0), NetworkSetback::None);
    assert_eq!(network_profile_setback(1), NetworkSetback::Failure);
    assert_eq!(network_profile_setback(6_000), NetworkSetback::SevereLoss);
}

#[test]
fn hard_host_connection_limit_caps_the_configured_ceiling() {
    assert_eq!(connection_ceiling(3, 0), 3);
    assert_eq!(connection_ceiling(3, 1), 1);
}
