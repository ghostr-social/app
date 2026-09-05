# WARP v3 acceptance map

The complete physical matrix, Flutter suite, native suite, and Axiom pass as
recorded in the [audit](WARP_V3_IMPLEMENTATION_STATUS.md). These are the
concrete regression entry points, not a claim that every Cartesian combination
of media, device, network, and failure has been tested.

| Case | Regression entry point | Scope |
|---|---|---|
| H01 | [chunk_downloader_partial_content_test.rs](../rust/crates/delivery/tests/chunk_downloader_partial_content_test.rs) | Exact 206 extent |
| H02 | [chunk_downloader_range_ignored_test.rs](../rust/crates/delivery/tests/chunk_downloader_range_ignored_test.rs) | 200 starts at zero; renewed whole authority |
| H03 | [partial_range_generation_fence_test.rs](../rust/crates/partial-store/tests/partial_range_generation_fence_test.rs) | Changed source generation rejects stale assembly |
| H04 | [partial_range_validatorless_http_generation_reload_test.rs](../rust/crates/partial-store/tests/partial_range_validatorless_http_generation_reload_test.rs) | No validator-based resumed authority |
| H05 | [delivery_source_identity_race_test.rs](../rust/crates/delivery/tests/delivery_source_identity_race_test.rs), [probe_mirror_preserves_canonical_test.rs](../rust/crates/delivery/src/tests/probe_mirror_preserves_canonical_test.rs) | Endpoint identity remains independent; mirror metadata preserves canonical bytes |
| H06 | [chunk_downloader_full_body_cap_test.rs](../rust/crates/delivery/tests/chunk_downloader_full_body_cap_test.rs) | Unknown/oversized body cap |
| H07 | [chunk_downloader_short_body_test.rs](../rust/crates/delivery/tests/chunk_downloader_short_body_test.rs) | Truncated range remains incomplete |
| H08 | [chunk_downloader_identity_encoding_test.rs](../rust/crates/delivery/tests/chunk_downloader_identity_encoding_test.rs) | Content-coded ranges rejected |
| H09 | [progressive_range_opaque_head_bootstrap_journey_test.rs](../rust/crates/gateway/tests/progressive_range_opaque_head_bootstrap_journey_test.rs) | Playback bypasses unavailable HEAD |
| H10 | [chunk_downloader_rejected_status_header_observation_test.rs](../rust/crates/delivery/tests/chunk_downloader_rejected_status_header_observation_test.rs) | Rejected HTTP status observation; bounded recovery suites |
| I01 | [partial_range_action_mirror_staging_test.rs](../rust/crates/partial-store/tests/partial_range_action_mirror_staging_test.rs) | Mirror staging has separate authority |
| I02 | [integrity_claim_persistence_test.rs](../rust/crates/delivery/src/tests/integrity_claim_persistence_test.rs) | Scoped quarantine survives restart |
| I03 | [http_hls_gateway_redirect_base_test.rs](../rust/crates/gateway/tests/http_hls_gateway_redirect_base_test.rs) | Relative HLS descendants use their own base |
| I04 | [transformed_representation_identity_test.rs](../rust/crates/engine/src/tests/catalog/transformed_representation_identity_test.rs) | Transformed representation identity |
| M01 | [tail_moov_timeline_test.rs](../rust/crates/engine/src/tests/media_timeline/tail_moov_timeline_test.rs) | Tail structural metadata |
| M02 | [malformed_timeline_test.rs](../rust/crates/engine/src/tests/media_timeline/malformed_timeline_test.rs) | Malformed structural input cannot create readiness |
| M03 | [startup_composition_dependency_test.rs](../rust/crates/engine/src/tests/media_timeline/startup_composition_dependency_test.rs) | Track timing and random-access dependencies |
| M04 | [progressive_delivery_video_test.dart](../integration_test/progressive_delivery_video_test.dart) | Native MP4 backend; MSE excluded |
| M05 | [warp_feed_mixed_hls_readiness_video_test.dart](../integration_test/warp_feed_mixed_hls_readiness_video_test.dart) | Supported HLS dependency/player path |
| M06 | [http_hls_gateway_security_test.rs](../rust/crates/gateway/tests/http_hls_gateway_security_test.rs) | Prohibited descendant destinations |
| M07 | [warp_feed_unsupported_hls_rescue_video_test.dart](../integration_test/warp_feed_unsupported_hls_rescue_video_test.dart) | Typed unsupported profile and ordinary alternative |
| P01 | [video_player_lifecycle_contract_test.dart](../integration_test/video_player_lifecycle_contract_test.dart) | Native cold activation and first-frame evidence |
| P02 | [startup_continuation_cushion_test.rs](../rust/crates/engine/src/tests/media_timeline/startup_continuation_cushion_test.rs) | Continuation footprint is distinct from first frame |
| P03 | [startup_selected_track_duration_test.rs](../rust/crates/engine/src/tests/media_timeline/startup_selected_track_duration_test.rs) | Required-track coverage ends conservatively |
| P04 | [warp_feed_rapid_swipe_instrumentation_video_test.dart](../integration_test/warp_feed_rapid_swipe_instrumentation_video_test.dart) | Rapid navigation epoch ownership |
| P05 | [warp_feed_android_lifecycle_video_test.dart](../integration_test/warp_feed_android_lifecycle_video_test.dart) | Physical HOME/foreground handling |
| P06 | [warp_feed_adaptive_warm_back_video_test.dart](../integration_test/warp_feed_adaptive_warm_back_video_test.dart), [warp_feed_cache_pressure_video_test.dart](../integration_test/warp_feed_cache_pressure_video_test.dart), [warp_evicted_current_priority_test.rs](../rust/crates/engine/src/tests/adaptive/warp_evicted_current_priority_test.rs) | Backward intent, warm activation, and cold reacquisition before future downloads |
| B01 | [progressive_gateway_concurrent_demand_lease_test.rs](../rust/crates/gateway/tests/progressive_gateway_concurrent_demand_lease_test.rs) | Overlapping consumers share demands |
| B02 | [progressive_gateway_demand_release_test.rs](../rust/crates/gateway/tests/progressive_gateway_demand_release_test.rs) | Reference-counted demand release |
| B03 | [chunk_downloader_stall_timeout_test.rs](../rust/crates/delivery/tests/chunk_downloader_stall_timeout_test.rs) | Cancellation independent of body progress |
| B04 | [media_cumulative_allowance_test.rs](../rust/crates/net/tests/media_cumulative_allowance_test.rs) | Rate/control changes cannot manufacture allowance |
| B05 | [demanded_read_ahead_plan_test.rs](../rust/crates/delivery/src/tests/demanded_read_ahead_plan_test.rs) | Urgent bounded dependency slice |
| B06 | [partial_range_policy_generation_provenance_test.rs](../rust/crates/partial-store/tests/partial_range_policy_generation_provenance_test.rs), [delivery_manager_total_persistence_failure_test.rs](../rust/crates/delivery/tests/delivery_manager_total_persistence_failure_test.rs) | Crash recovery respects committed authority; failed storage bindings retry after repair |
| B07 | [partial_range_sparse_response_envelope_test.rs](../rust/crates/partial-store/tests/partial_range_sparse_response_envelope_test.rs) | Bounded sparse response envelope |
| B05/B06 | [whole_fallback_storage_pressure_test.rs](../rust/crates/engine/src/tests/adaptive/whole_fallback_storage_pressure_test.rs), [delivery_orphan_cache_eviction_test.rs](../rust/crates/delivery/tests/delivery_orphan_cache_eviction_test.rs), [cold_reclaim_test.rs](../rust/crates/partial-store/src/tests/cold_reclaim_test.rs) | Feasible whole fallback gets storage before admission; range bootstrap avoids optional-whole over-reservation; cold cache reclamation preserves working media and leases |
| B08 | [ffi_reset_media_access_test.rs](../rust/tests/ffi_reset_media_access_test.rs) | Logout revokes private playback URL |
| S01 | [shared_path_network_conditions_test.rs](../rust/crates/delivery/src/tests/shared_path_network_conditions_test.rs) | Origin estimate constrained by shared path |
| S02 | [watch_model_session_test.rs](../rust/crates/engine/tests/watch_model_session_test.rs) | Session-conditioned watch prediction |
| S03 | [warp_planner_action_frontier_test.rs](../rust/crates/engine/src/tests/adaptive/warp_planner_action_frontier_test.rs) | Dependency reachability; deterministic action frontier |
| S04 | [warp_planner_digital_twin_test.rs](../rust/crates/engine/src/tests/adaptive/warp_planner_digital_twin_test.rs) | Optional LOOKAHEAD model only; disabled in production |
| S05 | [warp_planner_control_policy_test.rs](../rust/crates/engine/src/tests/adaptive/warp_planner_control_policy_test.rs) | Optional auxiliary controller only; disabled in production |
| S06 | [timeline_coordinator_stale_result_test.rs](../rust/crates/delivery/src/tests/timeline_coordinator_stale_result_test.rs) | Stale background proposal cannot publish |
| X01 | [outbound_media_private_dns_test.rs](../rust/crates/net/tests/outbound_media_private_dns_test.rs) | Resolved public-address enforcement |
| X02 | [progressive_gateway_capability_required_test.rs](../rust/crates/gateway/tests/progressive_gateway_capability_required_test.rs) | Scoped loopback capability required |
| X03 | [mp4_resource_limit_test.rs](../rust/crates/engine/src/tests/media_timeline/mp4_resource_limit_test.rs) | Parser input/allocation/work bounds |
| R01 | [timeline_index_reuse_test.rs](../rust/crates/delivery/src/tests/timeline_index_reuse_test.rs) | Index survives payload eviction; changed source misses |
| R14 | [delivery_access_reset_test.rs](../rust/crates/delivery/tests/delivery_access_reset_test.rs) | Private buffers released; public payload retained |
| D01, D02, D04, D05 | [service_deficit_test.rs](../rust/crates/engine/src/tests/playback/service_deficit_test.rs) | Left limits, equal timestamps, continuous versus batched service |
| D03 | [startup_selected_track_duration_test.rs](../rust/crates/engine/src/tests/media_timeline/startup_selected_track_duration_test.rs) | Required-track contiguous coverage; shorter audio bounds video |
| D06 | [buffer_deficit_policy_test.rs](../rust/crates/engine/src/tests/playback/buffer_deficit_policy_test.rs), [rendition_buffer_downgrade_test.rs](../rust/crates/engine/src/tests/rendition/rendition_buffer_downgrade_test.rs) | Uncapped requirement and admissible rendition downgrade |
| D07 | [indexed_service_deficit_test.rs](../rust/crates/engine/src/tests/playback/indexed_service_deficit_test.rs) | Recompute conditional service from the remaining dependency set |
| D08 | [estimate_confidence_test.rs](../rust/crates/engine/src/tests/playback/estimate_confidence_test.rs), [release manifest](WARP_V3_RELEASE_MANIFEST.md) | Confidence aging and explicit limits; no measured Internet reliability claim |
| D09 | [service_deficit_consumption_test.rs](../rust/crates/engine/src/tests/playback/service_deficit_consumption_test.rs) | Pause, rate, and network-stall consumption |
| U04/U07 | [rendition_switch_state_reset_test.rs](../rust/crates/delivery/src/tests/rendition_switch_state_reset_test.rs) | Controlled rendition replacement and teardown |
| U05 | [warp_feed_playback_video_test.dart](../integration_test/warp_feed_playback_video_test.dart) | Ordinary Nostr media without WARP sidecars |

The remaining R, V, J, and U extension-specific cases are not applicable to this
CORE/3 implementation because their profiles are disabled. R01 and R14 apply to local
compiled indexes and are included above. Optional-controller unit tests do not
constitute permission to enable LOOKAHEAD/1.

Run `make native-test`, `make test-coverage`, and
`make video-android-physical-evidence ANDROID_PHYSICAL_SERIAL=22e0d933` for the
complete automated/device surfaces. `make axiom` is an additional required gate.
