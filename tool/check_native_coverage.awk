BEGIN {
  threshold["rust/src/api/accepted_events.rs"] = 95
  threshold["rust/src/api/broadcast_control.rs"] = 95
  threshold["rust/src/api/candidate_delivery.rs"] = 95
  threshold["rust/src/api/delivery_events_stream.rs"] = 95
  threshold["rust/src/api/debug_nostr.rs"] = 95
  threshold["rust/src/api/debug_relay_status.rs"] = 95
  threshold["rust/src/api/engine_control.rs"] = 95
  threshold["rust/src/api/event_control.rs"] = 95
  threshold["rust/src/api/event_runtime.rs"] = 95
  threshold["rust/src/api/event_snapshots.rs"] = 100
  threshold["rust/src/api/event_types.rs"] = 100
  threshold["rust/src/api/feed_control.rs"] = 95
  threshold["rust/src/api/feed_decisions.rs"] = 100
  threshold["rust/src/api/feed_mapping.rs"] = 100
  threshold["rust/src/api/feed_outcome_pump.rs"] = 95
  threshold["rust/src/api/feed_outcomes.rs"] = 95
  threshold["rust/src/api/feed_progress.rs"] = 100
  threshold["rust/src/api/feed_projection.rs"] = 100
  threshold["rust/src/api/feed_runtime.rs"] = 95
  threshold["rust/src/api/feed_runtime_start.rs"] = 95
  threshold["rust/src/api/feed_state.rs"] = 100
  threshold["rust/src/api/feed_state/session.rs"] = 100
  # The only uncovered statements are the FRB StreamSink adapter, which
  # requires a live Dart message port. The watcher/state behavior is fully
  # exercised; keep its measured floor explicit instead of hiding the file.
  threshold["rust/src/api/feed_updates_stream.rs"] = 88
  threshold["rust/src/api/focus_control.rs"] = 95
  threshold["rust/src/api/focus_mapping.rs"] = 100
  threshold["rust/src/api/runtime_configuration.rs"] = 95
  threshold["rust/src/api/runtime_registry.rs"] = 95
  threshold["rust/src/api/session_control.rs"] = 95
  threshold["rust/src/api/tracked_items.rs"] = 100

  threshold["rust/crates/discovery/src/cache_fallback.rs"] = 100
  threshold["rust/crates/discovery/src/candidate_registry.rs"] = 100
  threshold["rust/crates/discovery/src/control_loop.rs"] = 100
  threshold["rust/crates/discovery/src/discovery_scheduler.rs"] = 95
  threshold["rust/crates/discovery/src/discovery_scheduler/handle.rs"] = 95
  threshold["rust/crates/discovery/src/event_cache.rs"] = 95
  threshold["rust/crates/discovery/src/event_cache_database.rs"] = 95
  threshold["rust/crates/discovery/src/event_cache_merge.rs"] = 100
  threshold["rust/crates/discovery/src/event_cache_session.rs"] = 100
  threshold["rust/crates/discovery/src/event_parsing.rs"] = 100
  threshold["rust/crates/discovery/src/event_queries.rs"] = 100
  threshold["rust/crates/discovery/src/feed_assembly.rs"] = 100
  threshold["rust/crates/discovery/src/feed_cursor.rs"] = 100
  threshold["rust/crates/discovery/src/feed_spec.rs"] = 100
  threshold["rust/crates/discovery/src/feed_store.rs"] = 100
  threshold["rust/crates/discovery/src/hashtags.rs"] = 100
  threshold["rust/crates/discovery/src/live_search_relays.rs"] = 95
  threshold["rust/crates/discovery/src/outbox_bootstrap.rs"] = 95
  threshold["rust/crates/discovery/src/outbox_directory.rs"] = 100
  threshold["rust/crates/discovery/src/outbox_plans.rs"] = 100
  threshold["rust/crates/discovery/src/outbox_relay_list.rs"] = 100
  threshold["rust/crates/discovery/src/pagination.rs"] = 100
  threshold["rust/crates/discovery/src/plan_executor.rs"] = 100
  threshold["rust/crates/discovery/src/profile_store.rs"] = 100
  threshold["rust/crates/discovery/src/relay_fetch.rs"] = 95
  threshold["rust/crates/discovery/src/relay_io.rs"] = 95
  threshold["rust/crates/discovery/src/relay_plan_collector.rs"] = 95
  threshold["rust/crates/discovery/src/relay_plan_executor.rs"] = 95
  threshold["rust/crates/discovery/src/relay_plan_executor/profile_enrichment.rs"] = 95
  threshold["rust/crates/discovery/src/relay_plan_routes.rs"] = 100
  threshold["rust/crates/discovery/src/relay_removal.rs"] = 95
  threshold["rust/crates/discovery/src/relay_pool_owner.rs"] = 95
  threshold["rust/crates/discovery/src/relay_pool_roles.rs"] = 100
  threshold["rust/crates/discovery/src/relay_pool_route.rs"] = 95
  threshold["rust/crates/discovery/src/relay_pool_transition.rs"] = 95
  threshold["rust/crates/discovery/src/relay_url.rs"] = 100
  threshold["rust/crates/discovery/src/retrieval_queue.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler_commands.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler_feeds.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler_loop.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler_plans.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler_queries.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler_session.rs"] = 95
  threshold["rust/crates/discovery/src/search_queries.rs"] = 100
  threshold["rust/crates/discovery/src/session_generation.rs"] = 100
  threshold["rust/crates/discovery/src/social_graph.rs"] = 100
  threshold["rust/crates/discovery/src/trending.rs"] = 100
  threshold["rust/crates/discovery/src/video_filters.rs"] = 100

  threshold["rust/crates/engine/src/budget.rs"] = 100
  threshold["rust/crates/engine/src/catalog.rs"] = 100
  threshold["rust/crates/engine/src/chunk_plan.rs"] = 100
  threshold["rust/crates/engine/src/focus.rs"] = 100
  threshold["rust/crates/engine/src/host_stats.rs"] = 100
  threshold["rust/crates/engine/src/host_stats_persistence.rs"] = 95
  threshold["rust/crates/engine/src/inventory_controller.rs"] = 100
  threshold["rust/crates/engine/src/scoring.rs"] = 100
  threshold["rust/crates/engine/src/tiers.rs"] = 100

  threshold["rust/crates/delivery/src/cache_registry.rs"] = 100
  threshold["rust/crates/delivery/src/candidate_priority.rs"] = 100
  threshold["rust/crates/delivery/src/chunk_cancel.rs"] = 100
  threshold["rust/crates/delivery/src/chunk_downloader.rs"] = 95
  threshold["rust/crates/delivery/src/chunk_network.rs"] = 95
  threshold["rust/crates/delivery/src/chunk_response.rs"] = 100
  threshold["rust/crates/delivery/src/chunk_stream.rs"] = 95
  threshold["rust/crates/net/src/content_range.rs"] = 100
  threshold["rust/crates/gateway/src/debug_http.rs"] = 95
  threshold["rust/crates/delivery/src/debug_feed.rs"] = 100
  threshold["rust/crates/delivery/src/debug_network.rs"] = 95
  threshold["rust/crates/gateway/src/debug_state.rs"] = 95
  threshold["rust/crates/gateway/src/debug_videos.rs"] = 95
  threshold["rust/crates/delivery/src/delivery_completion.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_cache.rs"] = 95
  threshold["rust/crates/delivery/src/delivery_events.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_failure.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_inflight.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_manager.rs"] = 95
  threshold["rust/crates/delivery/src/delivery_plan.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_probe_completion.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_pressure.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_reconcile.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_retry.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_state.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_stats.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_transfers.rs"] = 95
  threshold["rust/crates/delivery/src/delivery_wake.rs"] = 95
  threshold["rust/crates/delivery/src/download_workers.rs"] = 95
  threshold["rust/crates/media-model/src/event_identity.rs"] = 100
  threshold["rust/src/video/ffi_models.rs"] = 95
  threshold["rust/crates/gateway/src/gateway_delivery.rs"] = 95
  threshold["rust/crates/gateway/src/gateway_runtime.rs"] = 95
  threshold["rust/crates/gateway/src/hls_http_gateway.rs"] = 95
  threshold["rust/crates/hls-manifest/src/hls_manifest.rs"] = 100
  threshold["rust/crates/hls-manifest/src/hls_manifest_attributes.rs"] = 100
  threshold["rust/crates/hls-manifest/src/hls_manifest_tags.rs"] = 100
  threshold["rust/crates/gateway/src/hls_playback_gateway.rs"] = 95
  threshold["rust/crates/gateway/src/hls_resource_capability.rs"] = 100
  threshold["rust/crates/gateway/src/hls_session_state.rs"] = 95
  threshold["rust/crates/gateway/src/hls_session_types.rs"] = 100
  threshold["rust/crates/gateway/src/hls_sessions.rs"] = 95
  threshold["rust/crates/gateway/src/http_gateway.rs"] = 95
  threshold["rust/crates/media-model/src/imeta_extras.rs"] = 100
  threshold["rust/crates/delivery/src/media_probe.rs"] = 95
  threshold["rust/crates/delivery/src/metadata_probe_pool.rs"] = 100
  threshold["rust/crates/delivery/src/mp4_moov.rs"] = 100
  threshold["rust/crates/delivery/src/mutable_priority_queue.rs"] = 100
  threshold["rust/crates/media-store/src/native_blob_integrity.rs"] = 95
  threshold["rust/crates/media-store/src/native_blob_store.rs"] = 95
  threshold["rust/crates/media-store/src/native_cache.rs"] = 95
  threshold["rust/crates/media-store/src/native_cache_capacity.rs"] = 100
  threshold["rust/crates/media-store/src/native_cache_digest.rs"] = 100
  threshold["rust/crates/media-store/src/native_cache_directory.rs"] = 100
  threshold["rust/crates/net/src/native_cache_failure.rs"] = 100
  threshold["rust/crates/media-store/src/native_cache_fetch.rs"] = 100
  threshold["rust/crates/media-store/src/native_cache_transfer.rs"] = 95
  threshold["rust/crates/media-model/src/native_download_state.rs"] = 100
  threshold["rust/src/video/native_gateway.rs"] = 95
  threshold["rust/crates/media-model/src/native_media_metadata.rs"] = 100
  threshold["rust/crates/media-model/src/native_models.rs"] = 100
  threshold["rust/crates/media-store/src/native_partial_store.rs"] = 95
  threshold["rust/crates/media-model/src/native_text.rs"] = 100
  threshold["rust/crates/media-model/src/nostr_event_media.rs"] = 100
  threshold["rust/crates/net/src/origin_content_type.rs"] = 100
  threshold["rust/crates/media-store/src/partial_range_completion.rs"] = 100
  threshold["rust/crates/net/src/outbound_media_client.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_disk.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_manifest.rs"] = 100
  threshold["rust/crates/media-store/src/partial_range_paths.rs"] = 100
  threshold["rust/crates/media-store/src/partial_range_store.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_store/admission.rs"] = 100
  threshold["rust/crates/media-store/src/partial_range_store/capacity.rs"] = 100
  threshold["rust/crates/media-store/src/partial_range_store/eviction.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_store/finalize.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_store/free_space.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_store/leases.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_store/queries.rs"] = 100
  threshold["rust/crates/media-store/src/partial_range_store/reload.rs"] = 95
  threshold["rust/crates/media-store/src/partial_range_store/writes.rs"] = 95
  threshold["rust/crates/delivery/src/playback_demand.rs"] = 100
  threshold["rust/crates/media-model/src/post_text.rs"] = 100
  threshold["rust/crates/gateway/src/progressive_route.rs"] = 95
  threshold["rust/crates/gateway/src/progressive_stream.rs"] = 95
  threshold["rust/crates/net/src/public_dns_resolver.rs"] = 95
  threshold["rust/crates/net/src/public_media_address.rs"] = 100
  threshold["rust/crates/gateway/src/range_header.rs"] = 100
  threshold["rust/crates/net/src/transfer_timeouts.rs"] = 100
  threshold["rust/crates/media-model/src/video_link_scan.rs"] = 100
}

function canonical_source(raw, marker) {
  gsub(/\\/, "/", raw)
  sub(/\r$/, "", raw)
  if (substr(raw, 1, 9) == "rust/src/" || substr(raw, 1, 12) == "rust/crates/") {
    return raw
  }
  if (substr(raw, 1, 4) == "src/" || substr(raw, 1, 7) == "crates/") {
    return "rust/" raw
  }
  marker = index(raw, "/rust/src/")
  if (marker == 0) {
    marker = index(raw, "/rust/crates/")
  }
  return marker == 0 ? raw : substr(raw, marker + 1)
}

/^SF:/ {
  source = canonical_source(substr($0, 4))
  hit = 0
  total = 0
  next
}

/^DA:/ && source in threshold {
  split(substr($0, 4), data, ",")
  total++
  if (data[2] > 0) {
    hit++
  }
  next
}

/^end_of_record/ && source in threshold {
  seen[source] = 1
  coverage = total == 0 ? 0 : hit * 100 / total
  printf("Native line coverage %s: %.2f%% (%d/%d; required %.0f%%)\n",
         source, coverage, hit, total, threshold[source])
  if (coverage + 0.000001 < threshold[source]) {
    failed = 1
  }
}

END {
  for (required in threshold) {
    if (!(required in seen)) {
      printf("Missing native coverage record for %s\n", required)
      failed = 1
    }
  }
  exit failed
}
