BEGIN {
  threshold["rust/src/api/accepted_events.rs"] = 95
  threshold["rust/src/api/broadcast_control.rs"] = 95
  threshold["rust/src/api/delivery_events_stream.rs"] = 95
  threshold["rust/src/api/engine_control.rs"] = 95
  threshold["rust/src/api/event_control.rs"] = 95
  threshold["rust/src/api/event_runtime.rs"] = 95
  threshold["rust/src/api/event_snapshots.rs"] = 100
  threshold["rust/src/api/event_types.rs"] = 100
  threshold["rust/src/api/feed_control.rs"] = 95
  threshold["rust/src/api/feed_decisions.rs"] = 100
  threshold["rust/src/api/feed_mapping.rs"] = 100
  threshold["rust/src/api/feed_outcomes.rs"] = 95
  threshold["rust/src/api/feed_progress.rs"] = 100
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

  threshold["rust/src/discovery/cache_fallback.rs"] = 100
  threshold["rust/src/discovery/control_loop.rs"] = 100
  threshold["rust/src/discovery/discovery_scheduler.rs"] = 95
  threshold["rust/src/discovery/discovery_scheduler/handle.rs"] = 95
  threshold["rust/src/discovery/event_cache.rs"] = 95
  threshold["rust/src/discovery/event_cache_database.rs"] = 95
  threshold["rust/src/discovery/event_cache_merge.rs"] = 100
  threshold["rust/src/discovery/event_cache_session.rs"] = 100
  threshold["rust/src/discovery/event_parsing.rs"] = 100
  threshold["rust/src/discovery/event_queries.rs"] = 100
  threshold["rust/src/discovery/feed_assembly.rs"] = 100
  threshold["rust/src/discovery/feed_cursor.rs"] = 100
  threshold["rust/src/discovery/feed_spec.rs"] = 100
  threshold["rust/src/discovery/feed_store.rs"] = 100
  threshold["rust/src/discovery/hashtags.rs"] = 100
  threshold["rust/src/discovery/live_search_relays.rs"] = 95
  threshold["rust/src/discovery/outbox_bootstrap.rs"] = 95
  threshold["rust/src/discovery/outbox_directory.rs"] = 100
  threshold["rust/src/discovery/outbox_plans.rs"] = 100
  threshold["rust/src/discovery/outbox_relay_list.rs"] = 100
  threshold["rust/src/discovery/pagination.rs"] = 100
  threshold["rust/src/discovery/plan_executor.rs"] = 100
  threshold["rust/src/discovery/profile_store.rs"] = 100
  threshold["rust/src/discovery/relay_fetch.rs"] = 95
  threshold["rust/src/discovery/relay_io.rs"] = 95
  threshold["rust/src/discovery/relay_plan_collector.rs"] = 95
  threshold["rust/src/discovery/relay_plan_executor.rs"] = 95
  threshold["rust/src/discovery/relay_plan_executor/profile_enrichment.rs"] = 95
  threshold["rust/src/discovery/relay_plan_routes.rs"] = 100
  threshold["rust/src/discovery/relay_removal.rs"] = 95
  threshold["rust/src/discovery/relay_pool_owner.rs"] = 95
  threshold["rust/src/discovery/relay_pool_roles.rs"] = 100
  threshold["rust/src/discovery/relay_pool_route.rs"] = 95
  threshold["rust/src/discovery/relay_pool_transition.rs"] = 95
  threshold["rust/src/discovery/relay_url.rs"] = 100
  threshold["rust/src/discovery/retrieval_queue.rs"] = 100
  threshold["rust/src/discovery/scheduler_commands.rs"] = 100
  threshold["rust/src/discovery/scheduler_feeds.rs"] = 100
  threshold["rust/src/discovery/scheduler_loop.rs"] = 95
  threshold["rust/src/discovery/scheduler_plans.rs"] = 100
  threshold["rust/src/discovery/scheduler_queries.rs"] = 95
  threshold["rust/src/discovery/scheduler_session.rs"] = 95
  threshold["rust/src/discovery/search_queries.rs"] = 100
  threshold["rust/src/discovery/session_generation.rs"] = 100
  threshold["rust/src/discovery/social_graph.rs"] = 100
  threshold["rust/src/discovery/trending.rs"] = 100
  threshold["rust/src/discovery/video_filters.rs"] = 100

  threshold["rust/src/engine/budget.rs"] = 100
  threshold["rust/src/engine/catalog.rs"] = 100
  threshold["rust/src/engine/chunk_plan.rs"] = 100
  threshold["rust/src/engine/focus.rs"] = 100
  threshold["rust/src/engine/host_stats.rs"] = 100
  threshold["rust/src/engine/host_stats_persistence.rs"] = 95
  threshold["rust/src/engine/inventory_controller.rs"] = 100
  threshold["rust/src/engine/scoring.rs"] = 100
  threshold["rust/src/engine/tiers.rs"] = 100

  threshold["rust/src/video/chunk_cancel.rs"] = 100
  threshold["rust/src/video/chunk_downloader.rs"] = 95
  threshold["rust/src/video/chunk_response.rs"] = 100
  threshold["rust/src/video/content_range.rs"] = 100
  threshold["rust/src/video/delivery_completion.rs"] = 100
  threshold["rust/src/video/delivery_events.rs"] = 100
  threshold["rust/src/video/delivery_failure.rs"] = 100
  threshold["rust/src/video/delivery_inflight.rs"] = 100
  threshold["rust/src/video/delivery_manager.rs"] = 95
  threshold["rust/src/video/delivery_plan.rs"] = 100
  threshold["rust/src/video/delivery_probe_completion.rs"] = 100
  threshold["rust/src/video/delivery_probes.rs"] = 100
  threshold["rust/src/video/delivery_pressure.rs"] = 100
  threshold["rust/src/video/delivery_reconcile.rs"] = 100
  threshold["rust/src/video/delivery_retry.rs"] = 100
  threshold["rust/src/video/delivery_state.rs"] = 100
  threshold["rust/src/video/delivery_stats.rs"] = 100
  threshold["rust/src/video/delivery_transfers.rs"] = 95
  threshold["rust/src/video/event_identity.rs"] = 100
  threshold["rust/src/video/ffi_models.rs"] = 95
  threshold["rust/src/video/gateway_delivery.rs"] = 95
  threshold["rust/src/video/gateway_runtime.rs"] = 95
  threshold["rust/src/video/hls_http_gateway.rs"] = 95
  threshold["rust/src/video/hls_manifest.rs"] = 100
  threshold["rust/src/video/hls_manifest_attributes.rs"] = 100
  threshold["rust/src/video/hls_manifest_tags.rs"] = 100
  threshold["rust/src/video/hls_playback_gateway.rs"] = 95
  threshold["rust/src/video/hls_resource_capability.rs"] = 100
  threshold["rust/src/video/hls_session_state.rs"] = 95
  threshold["rust/src/video/hls_session_types.rs"] = 100
  threshold["rust/src/video/hls_sessions.rs"] = 95
  threshold["rust/src/video/http_gateway.rs"] = 95
  threshold["rust/src/video/imeta_extras.rs"] = 100
  threshold["rust/src/video/media_probe.rs"] = 95
  threshold["rust/src/video/mp4_moov.rs"] = 100
  threshold["rust/src/video/native_blob_integrity.rs"] = 95
  threshold["rust/src/video/native_blob_store.rs"] = 95
  threshold["rust/src/video/native_cache.rs"] = 95
  threshold["rust/src/video/native_cache_capacity.rs"] = 100
  threshold["rust/src/video/native_cache_digest.rs"] = 100
  threshold["rust/src/video/native_cache_directory.rs"] = 100
  threshold["rust/src/video/native_cache_failure.rs"] = 100
  threshold["rust/src/video/native_cache_fetch.rs"] = 100
  threshold["rust/src/video/native_cache_transfer.rs"] = 95
  threshold["rust/src/video/native_download_state.rs"] = 100
  threshold["rust/src/video/native_gateway.rs"] = 95
  threshold["rust/src/video/native_media_metadata.rs"] = 100
  threshold["rust/src/video/native_models.rs"] = 100
  threshold["rust/src/video/native_partial_store.rs"] = 95
  threshold["rust/src/video/native_text.rs"] = 100
  threshold["rust/src/video/nostr_event_media.rs"] = 100
  threshold["rust/src/video/origin_content_type.rs"] = 100
  threshold["rust/src/video/partial_range_completion.rs"] = 100
  threshold["rust/src/video/outbound_media_client.rs"] = 95
  threshold["rust/src/video/partial_range_disk.rs"] = 95
  threshold["rust/src/video/partial_range_manifest.rs"] = 100
  threshold["rust/src/video/partial_range_paths.rs"] = 100
  threshold["rust/src/video/partial_range_store.rs"] = 95
  threshold["rust/src/video/partial_range_store/admission.rs"] = 100
  threshold["rust/src/video/partial_range_store/capacity.rs"] = 100
  threshold["rust/src/video/partial_range_store/eviction.rs"] = 95
  threshold["rust/src/video/partial_range_store/finalize.rs"] = 95
  threshold["rust/src/video/partial_range_store/free_space.rs"] = 95
  threshold["rust/src/video/partial_range_store/leases.rs"] = 95
  threshold["rust/src/video/partial_range_store/queries.rs"] = 100
  threshold["rust/src/video/partial_range_store/reload.rs"] = 95
  threshold["rust/src/video/partial_range_store/writes.rs"] = 95
  threshold["rust/src/video/playback_demand.rs"] = 100
  threshold["rust/src/video/post_text.rs"] = 100
  threshold["rust/src/video/progressive_posts.rs"] = 100
  threshold["rust/src/video/progressive_route.rs"] = 95
  threshold["rust/src/video/progressive_stream.rs"] = 95
  threshold["rust/src/video/public_dns_resolver.rs"] = 95
  threshold["rust/src/video/public_media_address.rs"] = 100
  threshold["rust/src/video/range_header.rs"] = 100
  threshold["rust/src/video/transfer_timeouts.rs"] = 100
  threshold["rust/src/video/video_link_scan.rs"] = 100
}

function canonical_source(raw, marker) {
  gsub(/\\/, "/", raw)
  sub(/\r$/, "", raw)
  if (substr(raw, 1, 9) == "rust/src/") {
    return raw
  }
  if (substr(raw, 1, 4) == "src/") {
    return "rust/" raw
  }
  marker = index(raw, "/rust/src/")
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
