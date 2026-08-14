#!/bin/sh
set -eu

checker=tool/check_native_coverage.awk
failed=0

require_threshold() {
  actual=$(awk -v source="$1" \
    '$0 ~ "threshold\\[\\\"" source "\\\"\\]" { print $3 }' "$checker")
  if [ "$actual" != "$2" ]; then
    printf 'Expected native threshold %s for %s, found %s\n' \
      "$2" "$1" "${actual:-none}"
    failed=1
  fi
}

while IFS= read -r source; do
  require_threshold "$source" 100
done <<'EOF'
rust/crates/discovery/src/cache/session.rs
rust/crates/discovery/src/content/candidates.rs
rust/crates/discovery/src/content/deletion_index.rs
rust/crates/discovery/src/content/deletions.rs
rust/crates/discovery/src/content/pending_deletions.rs
rust/crates/discovery/src/content/repost_hint.rs
rust/crates/discovery/src/content/repost_reference.rs
rust/crates/discovery/src/content/repost_resolution.rs
rust/crates/discovery/src/content/reposts.rs
rust/crates/discovery/src/execution/relay_executor/deletion_hints.rs
rust/crates/discovery/src/execution/relay_executor/deletion_planning.rs
rust/crates/discovery/src/execution/relay_executor/deletion_targets.rs
rust/crates/discovery/src/execution/relay_executor/repost_retry.rs
rust/crates/discovery/src/execution/relay_executor/repost_support.rs
rust/crates/discovery/src/execution/relay_executor/target_dependencies.rs
rust/crates/discovery/src/execution/relay_executor/target_hints.rs
rust/crates/discovery/src/execution/relay_executor/target_planning.rs
rust/crates/discovery/src/feed/assembly.rs
rust/crates/discovery/src/feed/cursor.rs
rust/crates/discovery/src/feed/pagination.rs
rust/crates/discovery/src/feed/spec.rs
rust/crates/discovery/src/feed/store.rs
rust/crates/discovery/src/feed/store/occurrences.rs
rust/crates/discovery/src/feed/store/pages.rs
rust/crates/discovery/src/feed/store/progress.rs
rust/crates/discovery/src/feed/store_cursor.rs
rust/crates/discovery/src/feed/visibility.rs
rust/crates/discovery/src/query/video_filters.rs
rust/crates/discovery/src/query/events.rs
rust/crates/discovery/src/query/search.rs
rust/crates/discovery/src/relay/url.rs
rust/crates/discovery/src/retrieval_types.rs
rust/crates/discovery/src/scheduler/deferred_reposts.rs
rust/crates/discovery/src/scheduler/feeds.rs
rust/crates/discovery/src/scheduler/queue.rs
rust/src/api/delivery/candidates.rs
rust/src/api/feed/mapping.rs
rust/src/api/feed/state.rs
rust/src/api/feed/state/ingestion.rs
rust/src/api/feed/state/session.rs
EOF

while IFS= read -r source; do
  require_threshold "$source" 95
done <<'EOF'
rust/crates/discovery/src/cache.rs
rust/crates/discovery/src/execution/collector.rs
rust/crates/discovery/src/execution/fetch.rs
rust/crates/discovery/src/execution/relay_executor.rs
rust/crates/discovery/src/execution/relay_executor/deletion_enrichment.rs
rust/crates/discovery/src/execution/relay_executor/execution.rs
rust/crates/discovery/src/execution/relay_executor/fetches.rs
rust/crates/discovery/src/execution/relay_executor/profile_enrichment.rs
rust/crates/discovery/src/execution/relay_executor/target_enrichment.rs
rust/crates/discovery/src/outbox/bootstrap.rs
rust/crates/discovery/src/plan_executor.rs
rust/crates/discovery/src/relay/route.rs
rust/crates/discovery/src/scheduler.rs
rust/crates/discovery/src/scheduler/commands.rs
rust/crates/discovery/src/scheduler/event_loop.rs
rust/crates/discovery/src/scheduler/hunt.rs
rust/crates/discovery/src/scheduler/progress.rs
rust/crates/discovery/src/scheduler/retry.rs
rust/crates/discovery/src/scheduler/session.rs
rust/src/api/feed/outcome_pump.rs
rust/src/api/feed_control.rs
rust/src/api/runtime/discovery.rs
EOF

exit "$failed"
