use crate::manager::concurrency::{
    capacity_evidence, network_profile_setback, planning_connection_capacity, request_occupancy,
    HlsDemand, RequestConcurrencyLimits,
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
        1,
        10,
        Some(Duration::from_millis(120)),
    );

    let occupancy = request_occupancy(window, 2, 2);
    let evidence = capacity_evidence(window, true, Duration::from_secs(1), occupancy);

    assert_eq!(evidence.aggregate_bytes_per_second, 2_000_000);
    assert_eq!(
        evidence.occupancy,
        ConcurrencyOccupancy::new(1, 2).with_claimed_requests(2)
    );
    assert!(evidence.saturated);
    assert_eq!(evidence.ttfb, Duration::from_millis(120));
}

#[test]
fn observed_packet_loss_is_an_immediate_concurrency_setback() {
    assert_eq!(network_profile_setback(0), None);
    assert_eq!(network_profile_setback(1), Some(NetworkSetback::Failure));
    assert_eq!(
        network_profile_setback(6_000),
        Some(NetworkSetback::SevereLoss)
    );
}

#[test]
fn default_origin_limit_preserves_a_cross_origin_request_slot() {
    let inherited = RequestConcurrencyLimits::resolve(3, None, 0);
    let debug_limited = RequestConcurrencyLimits::resolve(3, None, 1);
    let configured = RequestConcurrencyLimits::resolve(3, NonZeroUsize::new(2), 0);

    assert_eq!(inherited.global(), 3);
    assert_eq!(inherited.per_authority(), 2);
    assert_eq!(debug_limited.global(), 3);
    assert_eq!(debug_limited.per_authority(), 1);
    assert_eq!(configured.per_authority(), 2);
}

#[test]
fn hls_demand_uses_healthy_global_capacity_but_respects_severe_loss() {
    assert_eq!(capacity(1, HlsDemand::new(3, true), 3, 0), 3);
    assert_eq!(capacity(1, HlsDemand::new(3, true), 2, 0), 2);
    assert_eq!(capacity(2, HlsDemand::new(1, true), 3, 0), 2);
    assert_eq!(capacity(2, HlsDemand::new(0, true), 3, 0), 2);
    assert_eq!(capacity(1, HlsDemand::new(5, true), 3, 0), 3);
    assert_eq!(capacity(1, HlsDemand::new(3, false), 3, 0), 1);
    assert_eq!(capacity(1, HlsDemand::new(3, true), 3, 6_000), 1);
}

fn capacity(adaptive: usize, hls: HlsDemand, ceiling: usize, loss: u16) -> usize {
    planning_connection_capacity(adaptive, hls, ceiling, loss)
}
