//! Scheduler test module index, split from `tests/mod.rs` to keep the test
//! inventory within the repository's file-size gate.

#[path = "discovery_scheduler_comfort_test.rs"]
mod discovery_scheduler_comfort_test;
#[path = "discovery_scheduler_concurrency_test.rs"]
mod discovery_scheduler_concurrency_test;
#[path = "discovery_scheduler_data_usage_test.rs"]
mod discovery_scheduler_data_usage_test;
#[path = "discovery_scheduler_focus_test.rs"]
mod discovery_scheduler_focus_test;
#[path = "discovery_scheduler_load_more_test.rs"]
mod discovery_scheduler_load_more_test;
#[path = "discovery_scheduler_playable_cursor_test.rs"]
mod discovery_scheduler_playable_cursor_test;
#[path = "discovery_scheduler_prefetch_test.rs"]
mod discovery_scheduler_prefetch_test;
#[path = "discovery_scheduler_priority_test.rs"]
mod discovery_scheduler_priority_test;
#[path = "discovery_scheduler_progress_test.rs"]
mod discovery_scheduler_progress_test;
#[path = "discovery_scheduler_query_close_test.rs"]
mod discovery_scheduler_query_close_test;
#[path = "discovery_scheduler_query_depth_test.rs"]
mod discovery_scheduler_query_depth_test;
#[path = "discovery_scheduler_query_refresh_test.rs"]
mod discovery_scheduler_query_refresh_test;
#[path = "discovery_scheduler_query_test.rs"]
mod discovery_scheduler_query_test;
#[path = "discovery_scheduler_reset_capacity_test.rs"]
mod discovery_scheduler_reset_capacity_test;
#[path = "discovery_scheduler_session_reset_test.rs"]
mod discovery_scheduler_session_reset_test;
#[path = "discovery_scheduler_stopped_test.rs"]
mod discovery_scheduler_stopped_test;
#[path = "discovery_scheduler_widen_test.rs"]
mod discovery_scheduler_widen_test;
#[path = "discovery_scheduler_wire_cursor_test.rs"]
mod discovery_scheduler_wire_cursor_test;
#[path = "scheduler_progress_bound_test.rs"]
mod scheduler_progress_bound_test;
#[path = "scheduler_support.rs"]
pub(super) mod scheduler_support;
#[path = "scheduler_wait.rs"]
mod scheduler_wait;
#[path = "scripted_scheduler_support.rs"]
mod scripted_scheduler_support;
