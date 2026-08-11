use crate::manager::concurrency::capacity_evidence;
use crate::manager::traffic::OverallTrafficWindow;
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

    let evidence = capacity_evidence(window, true, Duration::from_secs(1));

    assert_eq!(evidence.aggregate_bytes_per_second, 2_000_000);
    assert_eq!(evidence.active_transfers, 3);
    assert!(evidence.saturated);
    assert_eq!(evidence.ttfb, Duration::from_millis(120));
}
