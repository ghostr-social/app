use super::{progress, Active, SegmentedDelivery, SegmentedDone};
use std::sync::Arc;

pub(super) fn active_network() -> Arc<crate::segmented::fetch::FetchProgress> {
    Arc::new(crate::segmented::fetch::FetchProgress::default())
}

pub(super) fn finished_network() -> Arc<crate::segmented::fetch::FetchProgress> {
    let progress = active_network();
    progress.finish_network();
    progress
}

pub(super) fn test_fence(
    generation: u64,
    attempt: u64,
    url: &str,
    block_bytes: u64,
) -> crate::segmented::cache::StageFence {
    let request = crate::segmented::cache::StageRequest::new(url.to_owned(), 0, block_bytes);
    crate::segmented::cache::StageFence::new(generation, attempt, request)
}

pub(super) fn old_root_fence() -> crate::segmented::cache::StageFence {
    let bytes = ghostr_engine::adaptive::HlsBootstrapStage::RootManifest.maximum_bytes();
    test_fence(1, 1, "https://old.example/root.m3u8", bytes)
}

#[path = "cache_handoff_retention_test.rs"]
mod cache_handoff_retention_test;
#[path = "cache_pressure_reclaim_test.rs"]
mod cache_pressure_reclaim_test;
#[path = "cache_stale_reclaim_test.rs"]
mod cache_stale_reclaim_test;
#[path = "cancellation_race_test.rs"]
mod cancellation_race_test;
#[path = "cancellation_round_trip_race_test.rs"]
mod cancellation_round_trip_race_test;
#[path = "cancellation_success_fixture.rs"]
mod cancellation_success_fixture;
#[path = "cancellation_success_race_test.rs"]
mod cancellation_success_race_test;
#[path = "cancellation_test.rs"]
mod cancellation_test;
#[path = "focus_delivery_change_test.rs"]
mod focus_delivery_change_test;
#[path = "focus_root_cooldown_reset_test.rs"]
mod focus_root_cooldown_reset_test;
#[path = "focus_root_generation_revival_test.rs"]
mod focus_root_generation_revival_test;
#[path = "focus_root_order_test.rs"]
mod focus_root_order_test;
#[path = "focus_root_reconciliation_test.rs"]
mod focus_root_reconciliation_test;
#[path = "focus_root_selection_cursor_test.rs"]
mod focus_root_selection_cursor_test;
#[path = "invalidation_active_reseed_fixture.rs"]
mod invalidation_active_reseed_fixture;
#[path = "invalidation_active_reseed_test.rs"]
mod invalidation_active_reseed_test;
#[path = "priority_test.rs"]
mod priority_test;
#[path = "storage_reservation_fixture.rs"]
mod storage_reservation_fixture;
#[path = "storage_reservation_test.rs"]
mod storage_reservation_test;
#[path = "tests.rs"]
mod tests;
