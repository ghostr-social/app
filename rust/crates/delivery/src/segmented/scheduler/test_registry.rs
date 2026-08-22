use super::{progress, Active, SegmentedDelivery, SegmentedDone};

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
#[path = "storage_reservation_test.rs"]
mod storage_reservation_test;
#[path = "tests.rs"]
mod tests;
