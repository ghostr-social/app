BEGIN {
  # The coverage command enforces 95% globally. This map adds stricter local
  # gates only for the adaptive-delivery policy and its critical boundaries.
  threshold["rust/src/api/broadcast_control.rs"] = 95
  threshold["rust/src/api/delivery_events_stream.rs"] = 95
  threshold["rust/src/api/engine_control.rs"] = 95
  threshold["rust/src/api/event_control.rs"] = 95
  threshold["rust/src/api/event_types.rs"] = 100
  threshold["rust/src/api/feed_control.rs"] = 95
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
  threshold["rust/src/api/runtime/tracked_items.rs"] = 100

  threshold["rust/crates/engine/src/budget.rs"] = 100
  threshold["rust/crates/engine/src/catalog.rs"] = 100
  threshold["rust/crates/engine/src/chunk_plan.rs"] = 100
  threshold["rust/crates/engine/src/concurrency.rs"] = 100
  threshold["rust/crates/engine/src/concurrency/window.rs"] = 100
  threshold["rust/crates/engine/src/focus.rs"] = 100
  threshold["rust/crates/engine/src/host_stats.rs"] = 100
  threshold["rust/crates/engine/src/host_stats/evidence.rs"] = 100
  threshold["rust/crates/engine/src/host_stats/retention.rs"] = 100
  threshold["rust/crates/engine/src/inventory_controller.rs"] = 100
  threshold["rust/crates/engine/src/inventory_controller/startability.rs"] = 100
  threshold["rust/crates/engine/src/playback.rs"] = 100
  threshold["rust/crates/engine/src/playback/buffer.rs"] = 100
  threshold["rust/crates/engine/src/playback/network.rs"] = 100
  threshold["rust/crates/engine/src/playback/session.rs"] = 100
  threshold["rust/crates/engine/src/representation.rs"] = 100
  threshold["rust/crates/engine/src/scoring.rs"] = 100
  threshold["rust/crates/engine/src/scoring/frontier.rs"] = 100
  threshold["rust/crates/engine/src/tiers.rs"] = 100

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
  threshold["rust/crates/delivery/src/manager.rs"] = 95
  threshold["rust/crates/delivery/src/manager/completion.rs"] = 95
  threshold["rust/crates/delivery/src/manager/concurrency.rs"] = 100
  threshold["rust/crates/delivery/src/manager/inflight.rs"] = 95
  threshold["rust/crates/delivery/src/manager/plan.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan/playback.rs"] = 100
  threshold["rust/crates/delivery/src/manager/plan/startup.rs"] = 100
  threshold["rust/crates/delivery/src/manager/pressure.rs"] = 95
  threshold["rust/crates/delivery/src/manager/probe_completion.rs"] = 95
  threshold["rust/crates/delivery/src/manager/reconcile.rs"] = 100
  threshold["rust/crates/delivery/src/manager/retry.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/focus.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/playback.rs"] = 100
  threshold["rust/crates/delivery/src/manager/state/representation.rs"] = 100
  threshold["rust/crates/delivery/src/manager/stats.rs"] = 95
  threshold["rust/crates/delivery/src/manager/traffic.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/event.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/mailbox.rs"] = 95
  threshold["rust/crates/delivery/src/manager/traffic/mailbox/pending.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/timing.rs"] = 100
  threshold["rust/crates/delivery/src/manager/traffic/window.rs"] = 100
  threshold["rust/crates/delivery/src/manager/transfers.rs"] = 95
  threshold["rust/crates/delivery/src/manager/wake.rs"] = 95
  threshold["rust/crates/delivery/src/manager/wake_lane.rs"] = 100
  threshold["rust/crates/delivery/src/manager/wake_select.rs"] = 100
  threshold["rust/crates/delivery/src/mutable_priority_queue.rs"] = 100
  threshold["rust/crates/delivery/src/playback_demand.rs"] = 100

  threshold["rust/crates/gateway/src/delivery.rs"] = 95
  threshold["rust/crates/gateway/src/runtime.rs"] = 95
  # The sole excluded line retries a cryptographic 256-bit token collision;
  # malformed, expired, mismatched, released, reused, and evicted tokens are
  # covered through observable behavior without a test-only RNG seam.
  threshold["rust/crates/gateway/src/progressive/capabilities.rs"] = 99
  threshold["rust/crates/gateway/src/progressive/range_header.rs"] = 100
  threshold["rust/crates/gateway/src/progressive/route.rs"] = 95
  threshold["rust/crates/gateway/src/progressive/stream.rs"] = 95

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
