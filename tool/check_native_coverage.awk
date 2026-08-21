BEGIN {
  # The coverage command enforces 95% globally. This map adds stricter local
  # gates only for the adaptive-delivery policy and its critical boundaries.
  threshold["rust/src/api/broadcast_control.rs"] = 95
  threshold["rust/src/api/delivery_events_stream.rs"] = 95
  threshold["rust/src/api/engine_control.rs"] = 95
  threshold["rust/src/api/event_control.rs"] = 95
  threshold["rust/src/api/event_types.rs"] = 100
  threshold["rust/src/api/feed_control.rs"] = 95
  threshold["rust/src/api/feed/mapping.rs"] = 100
  threshold["rust/src/api/feed/outcome_pump.rs"] = 95
  threshold["rust/src/api/feed/state.rs"] = 100
  threshold["rust/src/api/feed/state/ingestion.rs"] = 100
  threshold["rust/src/api/feed/state/session.rs"] = 100
  # The uncovered statements are the FRB StreamSink adapter, which requires a
  # live Dart message port. Its watcher/state behavior is fully exercised.
  threshold["rust/src/api/feed_updates_stream.rs"] = 88
  threshold["rust/src/api/focus_control.rs"] = 95
  # The sole playback-control function is an FRB macro entry point and emits
  # no source region. Before-start failure and after-start delivery are both
  # exercised by integration tests, so no synthetic wrapper is introduced.
  threshold["rust/src/api/session_control.rs"] = 95
  threshold["rust/src/api/delivery/focus_mapping.rs"] = 100
  threshold["rust/src/api/delivery/playback_mapping.rs"] = 100
  threshold["rust/src/api/delivery/candidates.rs"] = 100
  threshold["rust/src/api/runtime/tracked_items.rs"] = 100
  threshold["rust/src/api/runtime/discovery.rs"] = 95
  threshold["rust/crates/engine/src/budget.rs"] = 100
  threshold["rust/crates/engine/src/catalog.rs"] = 100
  threshold["rust/crates/engine/src/catalog/bitrate.rs"] = 100
  threshold["rust/crates/engine/src/catalog/renditions.rs"] = 100
  threshold["rust/crates/engine/src/catalog/timeline.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/admission.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/allocation.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/allocation_evidence.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/catalog_snapshot.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/commitments.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/eviction.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/navigation.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/policy.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/ranges.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/resources.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/snapshot.rs"] = 100
  threshold["rust/crates/engine/src/adaptive/sources.rs"] = 100
  threshold["rust/crates/engine/src/concurrency.rs"] = 100
  threshold["rust/crates/engine/src/concurrency/occupancy.rs"] = 100
  threshold["rust/crates/engine/src/concurrency/trial.rs"] = 100
  threshold["rust/crates/engine/src/concurrency/window.rs"] = 100
  threshold["rust/crates/engine/src/focus.rs"] = 100
  threshold["rust/crates/engine/src/host_stats.rs"] = 100
  threshold["rust/crates/engine/src/host_stats/evidence.rs"] = 100
  threshold["rust/crates/engine/src/host_stats/retention.rs"] = 100
  threshold["rust/crates/engine/src/media_timeline.rs"] = 100
  threshold["rust/crates/engine/src/media_timeline/boxes.rs"] = 100
  threshold["rust/crates/engine/src/media_timeline/classic.rs"] = 100
  threshold["rust/crates/engine/src/media_timeline/classic/samples.rs"] = 100
  threshold["rust/crates/engine/src/media_timeline/classic/tables.rs"] = 100
  threshold["rust/crates/engine/src/media_timeline/sidx.rs"] = 100
  threshold["rust/crates/engine/src/playback.rs"] = 100
  threshold["rust/crates/engine/src/playback/buffer.rs"] = 100
  threshold["rust/crates/engine/src/playback/network.rs"] = 100
  threshold["rust/crates/engine/src/playback/session.rs"] = 100
  threshold["rust/crates/engine/src/representation.rs"] = 100
  threshold["rust/crates/engine/src/rendition/policy.rs"] = 100
  threshold["rust/crates/engine/src/rendition/risk.rs"] = 100
  threshold["rust/crates/engine/src/rendition/types.rs"] = 100
  threshold["rust/crates/engine/src/video_rendition.rs"] = 100
  threshold["rust/crates/delivery/src/cache_registry.rs"] = 100
  threshold["rust/crates/delivery/src/candidate_priority.rs"] = 100
  threshold["rust/crates/delivery/src/chunk/cancel.rs"] = 100
  threshold["rust/crates/delivery/src/chunk/downloader.rs"] = 95
  threshold["rust/crates/delivery/src/chunk/network.rs"] = 95
  threshold["rust/crates/delivery/src/chunk/response.rs"] = 100
  threshold["rust/crates/delivery/src/chunk/stream.rs"] = 95
  threshold["rust/crates/delivery/src/delivery_events.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_events/focus_generation.rs"] = 100
  threshold["rust/crates/delivery/src/delivery_events/mailbox.rs"] = 95
  threshold["rust/crates/delivery/src/delivery_events/mailbox/control.rs"] = 100
  threshold["rust/crates/delivery/src/debug/feed.rs"] = 95
  threshold["rust/crates/delivery/src/debug/feed/window.rs"] = 100
  threshold["rust/crates/delivery/src/debug/network.rs"] = 95
  threshold["rust/crates/delivery/src/debug/network/bandwidth.rs"] = 100
  threshold["rust/crates/delivery/src/manager.rs"] = 95
  threshold["rust/crates/delivery/src/manager/completion.rs"] = 95
  threshold["rust/crates/delivery/src/manager/concurrency.rs"] = 100
  threshold["rust/crates/delivery/src/manager/cooldown_timers.rs"] = 95
  threshold["rust/crates/delivery/src/manager/focus_lease.rs"] = 95
  threshold["rust/crates/delivery/src/manager/inflight.rs"] = 95
  threshold["rust/crates/delivery/src/manager/inflight/reconciliation.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan/adaptive.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan/adaptive/mapping.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan/adaptive/snapshot.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan/adaptive/telemetry.rs"] = 100
  threshold["rust/crates/delivery/src/manager/quality.rs"] = 95
  threshold["rust/crates/delivery/src/manager/pressure.rs"] = 95
  threshold["rust/crates/delivery/src/manager/probe_completion.rs"] = 95
  threshold["rust/crates/delivery/src/manager/reconcile.rs"] = 100
  threshold["rust/crates/delivery/src/manager/reset.rs"] = 95
  threshold["rust/crates/delivery/src/manager/retry.rs"] = 100
  threshold["rust/crates/delivery/src/manager/retry/cooldowns.rs"] = 100
  threshold["rust/crates/delivery/src/manager/retry/policy.rs"] = 100
  threshold["rust/crates/delivery/src/manager/retry_completion.rs"] = 95
  threshold["rust/crates/delivery/src/manager/state.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/evictions.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/focus.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/playback.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/probes.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/representation.rs"] = 100
  threshold["rust/crates/delivery/src/manager/stats.rs"] = 95
  threshold["rust/crates/delivery/src/manager/traffic.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/event.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/mailbox.rs"] = 95
  threshold["rust/crates/delivery/src/manager/traffic/mailbox/pending.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/timing.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/window.rs"] = 100
  threshold["rust/crates/delivery/src/manager/transfers.rs"] = 95
  threshold["rust/crates/delivery/src/manager/timeline.rs"] = 95
  threshold["rust/crates/delivery/src/manager/wake.rs"] = 95
  threshold["rust/crates/delivery/src/manager/wake_lane.rs"] = 100
  threshold["rust/crates/delivery/src/manager/wake_select.rs"] = 100
  threshold["rust/crates/delivery/src/manager/workers.rs"] = 95
  threshold["rust/crates/delivery/src/mutable_priority_queue.rs"] = 100
  threshold["rust/crates/delivery/src/playback_demand.rs"] = 100
  threshold["rust/crates/delivery/src/probe/pool.rs"] = 100
  threshold["rust/crates/discovery/src/cache.rs"] = 95
  threshold["rust/crates/discovery/src/cache/session.rs"] = 100
  threshold["rust/crates/discovery/src/content/candidates.rs"] = 100
  threshold["rust/crates/discovery/src/content/deletion_index.rs"] = 100
  threshold["rust/crates/discovery/src/content/deletions.rs"] = 100
  threshold["rust/crates/discovery/src/content/parsing.rs"] = 100
  threshold["rust/crates/discovery/src/content/pending_deletions.rs"] = 100
  threshold["rust/crates/discovery/src/content/renditions.rs"] = 100
  threshold["rust/crates/discovery/src/content/repost_hint.rs"] = 100
  threshold["rust/crates/discovery/src/content/repost_reference.rs"] = 100
  threshold["rust/crates/discovery/src/content/repost_resolution.rs"] = 100
  threshold["rust/crates/discovery/src/content/reposts.rs"] = 100
  threshold["rust/crates/discovery/src/execution/collector.rs"] = 95
  threshold["rust/crates/discovery/src/execution/fetch.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor/deletion_enrichment.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor/deletion_hints.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/deletion_planning.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/deletion_targets.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/execution.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor/fetches.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor/profile_enrichment.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor/repost_retry.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/repost_support.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/target_dependencies.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/target_enrichment.rs"] = 95
  threshold["rust/crates/discovery/src/execution/relay_executor/target_hints.rs"] = 100
  threshold["rust/crates/discovery/src/execution/relay_executor/target_planning.rs"] = 100
  threshold["rust/crates/discovery/src/feed/assembly.rs"] = 100
  threshold["rust/crates/discovery/src/feed/cursor.rs"] = 100
  threshold["rust/crates/discovery/src/feed/pagination.rs"] = 100
  threshold["rust/crates/discovery/src/feed/spec.rs"] = 100
  threshold["rust/crates/discovery/src/feed/store.rs"] = 100
  threshold["rust/crates/discovery/src/feed/store/occurrences.rs"] = 100
  threshold["rust/crates/discovery/src/feed/store/pages.rs"] = 100
  threshold["rust/crates/discovery/src/feed/store/progress.rs"] = 100
  threshold["rust/crates/discovery/src/feed/store_cursor.rs"] = 100
  threshold["rust/crates/discovery/src/feed/visibility.rs"] = 100
  threshold["rust/crates/discovery/src/outbox/bootstrap.rs"] = 95
  threshold["rust/crates/discovery/src/plan_executor.rs"] = 95
  threshold["rust/crates/discovery/src/query/events.rs"] = 100
  threshold["rust/crates/discovery/src/query/search.rs"] = 100
  threshold["rust/crates/discovery/src/query/video_filters.rs"] = 100
  threshold["rust/crates/discovery/src/relay/route.rs"] = 95
  threshold["rust/crates/discovery/src/relay/url.rs"] = 100
  threshold["rust/crates/discovery/src/retrieval_types.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler/commands.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler/deferred_reposts.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler/event_loop.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler/feeds.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler/hunt.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler/progress.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler/queue.rs"] = 100
  threshold["rust/crates/discovery/src/scheduler/retry.rs"] = 95
  threshold["rust/crates/discovery/src/scheduler/session.rs"] = 95
  threshold["rust/crates/gateway/src/delivery.rs"] = 95
  threshold["rust/crates/gateway/src/debug/http.rs"] = 95
  threshold["rust/crates/gateway/src/debug/state/video.rs"] = 95
  threshold["rust/crates/gateway/src/debug/videos.rs"] = 95
  threshold["rust/crates/gateway/src/runtime.rs"] = 95
  # The sole excluded line retries a cryptographic 256-bit token collision;
  # malformed, expired, mismatched, released, reused, and evicted tokens are
  # covered through observable behavior without a test-only RNG seam. The
  # 98.9% floor permits exactly that one line in the current 98-line adapter.
  threshold["rust/crates/gateway/src/progressive/capabilities.rs"] = 98.9
  threshold["rust/crates/gateway/src/progressive/range_header.rs"] = 100
  threshold["rust/crates/gateway/src/progressive/route.rs"] = 95
  threshold["rust/crates/gateway/src/progressive/route/snapshot.rs"] = 95
  threshold["rust/crates/gateway/src/progressive/stream.rs"] = 95
  threshold["rust/crates/gateway/src/progressive/stream/source.rs"] = 95
  threshold["rust/crates/media-model/src/imeta_extras.rs"] = 100
  threshold["rust/crates/media-model/src/native_media_metadata.rs"] = 100
  threshold["rust/crates/media-model/src/nostr_event_media.rs"] = 100
  threshold["rust/crates/net/src/content_range.rs"] = 100

  threshold["rust/crates/partial-store/src/partial_range_paths.rs"] = 100
  # LLVM assigns uncovered regions to two closing braces in these files. All
  # executable read, corruption, source-switch, and post-write race behavior
  # is covered; the measured floors avoid production reshaping for the map.
  threshold["rust/crates/partial-store/src/partial_range_representation_disk.rs"] = 95
  threshold["rust/crates/partial-store/src/partial_range_store.rs"] = 95
  threshold["rust/crates/partial-store/src/partial_range_store/admission.rs"] = 100
  threshold["rust/crates/partial-store/src/partial_range_store/capacity.rs"] = 100
  threshold["rust/crates/partial-store/src/partial_range_store/capacity/events.rs"] = 100
  threshold["rust/crates/partial-store/src/partial_range_store/leases.rs"] = 95
  threshold["rust/crates/partial-store/src/partial_range_store/representation.rs"] = 99
  threshold["rust/crates/partial-store/src/partial_range_store/writes.rs"] = 95
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
  printf("Native line coverage %s: %.2f%% (%d/%d; required %g%%)\n",
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
