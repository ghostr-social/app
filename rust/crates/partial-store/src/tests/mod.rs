#[path = "../../tests/store_fixture/busy_disk.rs"]
mod busy_disk_fixture;
#[path = "../../tests/compiled_index_fixture.rs"]
mod compiled_index_fixture;
#[path = "../../tests/compiled_index_retention_test.rs"]
mod compiled_index_retention_test;
mod external_a;
mod external_b;
mod external_c;
#[path = "../../tests/partial_range_action_prefix_conflict_test.rs"]
mod partial_range_action_prefix_conflict_test;
#[path = "../../tests/partial_range_action_prefix_disjoint_probe_test.rs"]
mod partial_range_action_prefix_disjoint_probe_test;
#[path = "../../tests/partial_range_action_prefix_existing_head_test.rs"]
mod partial_range_action_prefix_existing_head_test;
#[path = "../../tests/partial_range_action_prefix_sparse_fence_test.rs"]
mod partial_range_action_prefix_sparse_fence_test;
#[path = "../../tests/partial_range_action_scoped_prefix_completion_test.rs"]
mod partial_range_action_scoped_prefix_completion_test;
#[path = "../../tests/partial_range_action_scoped_prefix_corruption_test.rs"]
mod partial_range_action_scoped_prefix_corruption_test;
#[path = "../../tests/partial_range_action_scoped_prefix_identity_test.rs"]
mod partial_range_action_scoped_prefix_identity_test;
#[path = "../../tests/partial_range_action_scoped_prefix_restart_test.rs"]
mod partial_range_action_scoped_prefix_restart_test;
#[path = "../../tests/partial_range_action_scoped_prefix_revocation_test.rs"]
mod partial_range_action_scoped_prefix_revocation_test;
#[path = "../../tests/partial_range_action_scoped_prefix_test.rs"]
mod partial_range_action_scoped_prefix_test;
#[path = "../../tests/partial_range_cancel_before_eof_test.rs"]
mod partial_range_cancel_before_eof_test;
#[path = "../../tests/partial_range_durable_probe_prefix_test.rs"]
mod partial_range_durable_probe_prefix_test;
#[path = "../../tests/partial_range_durable_prefix_completion_test.rs"]
mod partial_range_durable_prefix_completion_test;
#[path = "../../tests/partial_range_durable_prefix_lease_test.rs"]
mod partial_range_durable_prefix_lease_test;
#[path = "../../tests/partial_range_durable_whole_cancellation_test.rs"]
mod partial_range_durable_whole_cancellation_test;
#[path = "../../tests/partial_range_transform_owner_failure_test.rs"]
mod partial_range_transform_owner_failure_test;
#[path = "../../tests/partial_range_warm_empty_snapshot_test.rs"]
mod partial_range_warm_empty_snapshot_test;
#[path = "../../tests/partial_range_warm_partial_snapshot_test.rs"]
mod partial_range_warm_partial_snapshot_test;
#[path = "../../tests/store_fixture/paused.rs"]
mod paused_fixture;
#[path = "../../tests/store_fixture/mod.rs"]
mod store_fixture;
mod system_free_space_test;
#[path = "../../tests/tail_recovery_fixture/mod.rs"]
mod tail_recovery_fixture;
#[path = "../../tests/transform_publication_fixture.rs"]
mod transform_publication_fixture;
#[path = "../../tests/transient_response_cancellation_test.rs"]
mod transient_response_cancellation_test;
#[path = "../../tests/transient_response_retention_test.rs"]
mod transient_response_retention_test;

mod cold_reclaim_test;
#[path = "../../tests/partial_range_selected_whole_cancellation_test.rs"]
mod partial_range_selected_whole_cancellation_test;
