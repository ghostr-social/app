# Adaptive Video Delivery Migration Guide

Status: migration contract for salvaging the current adaptive video delivery work.

## Decision

Do not merge or repair the `improve-architecture` branch as one unit. Create a
fresh branch from `origin/main` and transplant only the components listed here,
together with their focused behavioral tests.

Reuse behavior and safety properties, not commits, crate boundaries, or the
current orchestration. The branch contains two competing delivery control
planes and a current-only full-file policy that defeats parallelism and
prefetching.

## Migration rules

1. The delivery manager is the single owner of scheduling state.
2. Every migrated production behavior arrives with a failing characterization
   or acceptance test first.
3. Move one cohesive component at a time; do not cherry-pick the merge commit or
   broad checkpoint commits.
4. Preserve source identity, generation, cancellation, capacity, and stale-event
   protections while simplifying their APIs.
5. Keep product feed order separate from delivery readiness and scheduling.
6. A component is reusable only after its tests pass against the new single
   control plane.

## Reuse directly

The following code provides useful, bounded primitives. Transplant it in small
groups and retain its focused tests.

### Domain values and playback contracts

- Validated playback-buffer configuration and usage levels from
  [`rust/crates/video-policy/src/configuration.rs`](rust/crates/video-policy/src/configuration.rs).
- Media capability types from
  [`rust/crates/video-policy/src/media.rs`](rust/crates/video-policy/src/media.rs).
- Playback observation calculations from
  [`rust/crates/video-policy/src/playback.rs`](rust/crates/video-policy/src/playback.rs),
  after replacing placeholder runtime inputs with measured values.
- Playback session generation, observation sequencing, and stale-sample
  rejection from
  [`rust/crates/delivery/src/application/contracts/playback.rs`](rust/crates/delivery/src/application/contracts/playback.rs).
- Monotonic playback authorization from
  [`rust/crates/delivery/src/application/contracts/playback_authorization.rs`](rust/crates/delivery/src/application/contracts/playback_authorization.rs).
- Queue epoch, revision, exposure, and bounded replay-retention concepts from
  [`rust/crates/video-policy/src/queue.rs`](rust/crates/video-policy/src/queue.rs)
  and
  [`rust/crates/video-policy/src/queue/retention.rs`](rust/crates/video-policy/src/queue/retention.rs).

Keep these as one coherent domain vocabulary. Do not retain duplicate `PostId`,
`VideoId`, or `ByteRange` definitions across `engine` and `video-policy`.

### Ingress, cancellation, and concurrency safety

- Bounded candidate ingress and clear-time channel rotation from
  [`rust/crates/delivery/src/delivery_events/candidate_ingress.rs`](rust/crates/delivery/src/delivery_events/candidate_ingress.rs).
- Versioned, coalescing control delivery from
  [`rust/crates/delivery/src/delivery_events/control_mailbox.rs`](rust/crates/delivery/src/delivery_events/control_mailbox.rs).
- Clear-first and round-robin wake arbitration from
  [`rust/crates/delivery/src/manager/wake_select.rs`](rust/crates/delivery/src/manager/wake_select.rs)
  and
  [`rust/crates/delivery/src/manager/wake_lane.rs`](rust/crates/delivery/src/manager/wake_lane.rs).
- Preparation attempt fencing, cancellation, join barriers, and bounded
  total/per-host selection from
  [`rust/crates/delivery/src/manager/preparation_tasks/task_book.rs`](rust/crates/delivery/src/manager/preparation_tasks/task_book.rs)
  and
  [`rust/crates/delivery/src/manager/preparation_tasks/selection.rs`](rust/crates/delivery/src/manager/preparation_tasks/selection.rs).
- In-flight overlap rejection, cancellation, and ABA fencing from
  [`rust/crates/delivery/src/manager/inflight.rs`](rust/crates/delivery/src/manager/inflight.rs).
- Per-origin admission and priority preemption from
  [`rust/crates/net/src/network_control.rs`](rust/crates/net/src/network_control.rs).

These mechanisms should enforce decisions made by the new scheduler. They must
not become a second policy layer.

### Transport and persisted media

- Sparse ranges, leases, reservations, eviction notifications, and source
  epochs from
  [`rust/crates/partial-store/src/partial_range_store.rs`](rust/crates/partial-store/src/partial_range_store.rs).
- Representation/source validation from
  [`rust/crates/partial-store/src/partial_range_store/source.rs`](rust/crates/partial-store/src/partial_range_store/source.rs).
- Range request execution and response identity confirmation from
  [`rust/crates/delivery/src/chunk/downloader.rs`](rust/crates/delivery/src/chunk/downloader.rs)
  and
  [`rust/crates/delivery/src/chunk/downloader/opened.rs`](rust/crates/delivery/src/chunk/downloader/opened.rs).
- Cancellable, bounded, paced streaming and write-only-if-current behavior from
  [`rust/crates/delivery/src/chunk/stream.rs`](rust/crates/delivery/src/chunk/stream.rs)
  and
  [`rust/crates/delivery/src/chunk/sink.rs`](rust/crates/delivery/src/chunk/sink.rs).

Before migration is complete, make origin response classification require a
successful status. The current classifier treats any non-`206` response at
offset zero as a full body.

### Playback gateways

- Progressive HTTP Range parsing from
  [`rust/crates/gateway/src/progressive/range_header.rs`](rust/crates/gateway/src/progressive/range_header.rs).
- Store-backed progressive response streaming and decoder-demand signaling from
  [`rust/crates/gateway/src/progressive/stream.rs`](rust/crates/gateway/src/progressive/stream.rs).
- HLS session limits, signed resources, manifest rewriting, controlled asset
  proxying, and caching from
  [`rust/crates/gateway/src/hls/sessions.rs`](rust/crates/gateway/src/hls/sessions.rs),
  [`rust/crates/gateway/src/hls/routes.rs`](rust/crates/gateway/src/hls/routes.rs),
  and
  [`rust/crates/gateway/src/hls/routes/asset_route.rs`](rust/crates/gateway/src/hls/routes/asset_route.rs).

Add an opaque playback capability to progressive URLs, make HLS authorization
fail closed when binding is absent, and ensure a response never promises more
bytes than its body can deliver before an idle timeout.

### Flutter boundary

- Strong playback observations and sessions from
  [`lib/features/video_inventory/domain/playback_observation.dart`](lib/features/video_inventory/domain/playback_observation.dart)
  and
  [`lib/features/video_inventory/domain/playback_session.dart`](lib/features/video_inventory/domain/playback_session.dart).
- The narrow focus intent port from
  [`lib/features/video_catalog/domain/feed_focus_port.dart`](lib/features/video_catalog/domain/feed_focus_port.dart).
- Focus coalescing from
  [`lib/platform/media/ffi_feed_focus_scheduler.dart`](lib/platform/media/ffi_feed_focus_scheduler.dart).
- Session-aware, coalesced telemetry delivery from
  [`lib/platform/media/ffi_playback_telemetry_port.dart`](lib/platform/media/ffi_playback_telemetry_port.dart).
- Progressive and HLS gateway adapters from
  [`lib/platform/media/ffi_progressive_playback_gateway.dart`](lib/platform/media/ffi_progressive_playback_gateway.dart)
  and
  [`lib/platform/media/ffi_hls_playback_gateway.dart`](lib/platform/media/ffi_hls_playback_gateway.dart).

Keep telemetry reporting only for the active player. Define `stalled` from real
buffering/playback evidence; paused, ended, inactive, or merely not-playing are
not network stalls.

### Developer and E2E infrastructure

- Owned-process lifecycle and isolated run directories from
  [`tool/video_user_e2e/lifecycle.mjs`](tool/video_user_e2e/lifecycle.mjs),
  [`tool/video_user_e2e/owned_child.mjs`](tool/video_user_e2e/owned_child.mjs),
  and [`tool/video_user_e2e/run_files.mjs`](tool/video_user_e2e/run_files.mjs).
- Pinned browser/CDP session management and trusted visible input from
  [`tool/video_user_e2e/browser.mjs`](tool/video_user_e2e/browser.mjs),
  [`tool/video_user_e2e/cdp.mjs`](tool/video_user_e2e/cdp.mjs), and
  [`tool/video_user_e2e/page_runtime.mjs`](tool/video_user_e2e/page_runtime.mjs).
- Request ledgers and bounded failure artifacts from
  [`tool/video_user_e2e/request_ledger.mjs`](tool/video_user_e2e/request_ledger.mjs)
  and
  [`tool/video_user_e2e/trace_artifacts.mjs`](tool/video_user_e2e/trace_artifacts.mjs).

Use this plumbing for a deterministic local browser journey. Keep public-relay
testing as an optional repeated smoke or soak test, not the primary acceptance
gate.

## Reuse only as design input

The following ideas are useful, but their current implementations must not be
transplanted unchanged:

- Per-host TTFB, throughput, failure, and retry history. Retain the bounded
  persistence format, then add sample age/count, confidence, variance, and
  concurrency context.
- Progressive refill target and chunk-size calculations. Replace the single
  contiguous-prefix planner with explicit non-overlapping range intents.
- MP4 initialization and codec parsing. Use it to locate initialization and
  playable media geometry; a tail `moov` does not require a full download.
- Reservoir pressure and readiness scoring. Feed them actual playback rate,
  media bitrate, store usage, and aggregate network measurements.
- The Rust debug dashboard. Keep a thin read-only diagnostic surface, bound or
  virtualize its candidate list, add deterministic fixtures, and rename
  `make web` so it is not mistaken for the Flutter product.
- Fairness debt and host breadth. Preserve the public behavior with deterministic
  tests even if the private `GrantDebt` implementation is replaced.

## Do not migrate

- The merge commit or broad checkpoint commits from this branch.
- `DeliveryCoordinator` as a second candidate, queue, reservoir, and transfer
  authority alongside `DeliveryWorker`.
- The compatibility conversion in
  [`rust/crates/delivery/src/manager/policy_plan.rs`](rust/crates/delivery/src/manager/policy_plan.rs).
- The rule that the current ranged file must be complete before any ahead work.
- The one-range-per-post restriction and `TransferPlan` deduplication by video.
- `requires_complete_file` inferred from a tail `moov`.
- The native playback queue as the owner of product feed ordering or navigation.
- The second `VideoStore` accounting authority layered over the partial store.
- WebM capability claims while the verifier always returns `None`.
- Placeholder adaptation using fixed rates, unlimited storage, or a constant
  host factor.
- `stalled: !isPlaying` in Flutter or equivalent pause/end-as-stall logic in the
  debug browser.
- Source-text and regular-expression tests that assert implementation tokens
  instead of executing behavior.
- Tests rewritten or deleted solely to legitimize current-only delivery.
- Optional learned-policy/ML contracts until the deterministic evidence model
  and heuristic scheduler are complete and measured.

## Destination architecture

The migrated pipeline should have one direction of authority:

`product focus and playback observations`
→ `single manager-owned policy state machine`
→ `typed transfer/preparation intents`
→ `network admission and transport adapters`
→ `one storage authority`
→ `progressive/HLS gateway`
→ `player`

One transfer intent must identify the video, representation, source, source
generation, exact range or HLS resource, priority, and cancellation generation.
The scheduler may grant multiple disjoint ranges for one video when measured
parallelism is useful.

## Migration order

1. Add deterministic acceptance tests for current startup, bounded ahead
   prefetch, focus preemption, host fairness, parallel ranges, cancellation
   waste, and replay reuse.
2. Move the shared domain values and generation/session contracts.
3. Move the partial store, source identity, network admission, and transfer
   adapters with their protocol/race tests.
4. Move the progressive and HLS gateways after fixing fail-open and response-body
   correctness issues.
5. Implement one pure policy model and one manager-owned event loop.
6. Connect Flutter through focus, readiness, activation, and telemetry ports;
   keep the product roster Flutter-owned.
7. Build a deterministic local debug/E2E fixture, then add optional live-relay
   smoke coverage.
8. Remove all compatibility and shadow-orchestration code once the migrated
   path passes the full verification surface.

## Required acceptance behavior

Migration is complete only when automated tests prove all of the following:

- The current item becomes playable from a justified prefix, without requiring
  EOF for ordinary ranged progressive media.
- Ahead preparation begins after the current item has a safe measured buffer,
  before the current file is complete.
- A new focus preempts obsolete work promptly while retaining useful completed
  sparse ranges.
- Multiple disjoint current ranges may overlap in time when aggregate measured
  throughput improves.
- Slow or failing hosts cannot block eligible work on healthy hosts.
- Concurrency rises only after measured aggregate gain and falls on stall,
  failure, or material RTT inflation.
- Pauses and normal endings do not trigger network-emergency policy.
- Storage, cancellation waste, speculative bytes, and replay retention remain
  bounded.
- A deterministic browser test observes real `playing`, increasing media time,
  bounded stalls, post-safe ahead work, and reprioritization through visible
  controls.
