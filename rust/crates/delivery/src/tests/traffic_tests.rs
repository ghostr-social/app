use super::support;

#[path = "traffic_closed_interval_wake_test.rs"]
mod closed_interval_wake;
#[path = "traffic_mailbox_coalescing_test.rs"]
mod mailbox_coalescing;
#[path = "traffic_meter_aggregate_test.rs"]
mod meter_aggregate;
#[path = "traffic_meter_idempotence_test.rs"]
mod meter_idempotence;
#[path = "traffic_meter_staggered_host_test.rs"]
mod meter_staggered_host;
#[path = "traffic_silence_timer_test.rs"]
mod silence_timer;
#[path = "traffic_window_observability_test.rs"]
mod window_observability;
#[path = "traffic_zero_byte_host_test.rs"]
mod zero_byte_host;
#[path = "traffic_body_renewal_pause_test.rs"]
mod body_renewal_pause;
