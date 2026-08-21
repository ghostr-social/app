use crate::manager::concurrency::{
    capacity_evidence, network_profile_setback, RequestConcurrencyLimits,
};
use crate::manager::traffic::OverallTrafficWindow;
use ghostr_engine::concurrency::{ConcurrencyOccupancy, NetworkSetback};
use std::num::NonZeroUsize;
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
fn per_host_limit_does_not_collapse_the_global_ceiling() {
    let inherited = RequestConcurrencyLimits::resolve(3, None, 0);
    let debug_limited = RequestConcurrencyLimits::resolve(3, None, 1);
    let configured = RequestConcurrencyLimits::resolve(3, NonZeroUsize::new(2), 0);

    assert_eq!(inherited.global(), 3);
    assert_eq!(inherited.per_authority(), 3);
    assert_eq!(debug_limited.global(), 3);
    assert_eq!(debug_limited.per_authority(), 1);
    assert_eq!(debug_limited.segmented_compatibility(), 1);
    assert_eq!(configured.per_authority(), 2);
}
