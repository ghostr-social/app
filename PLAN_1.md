# PLAN 1 — Repair the current retrieval path

## Goal

Repair the existing production retrieval path: start the first video promptly,
prepare the immediate next video, keep playback observations connected to the
Rust policy, and extend the feed at the tail without another page change.

Work only in the existing feed—including the route and surface wiring that
mounts feed viewers and publishes focus—playback-observation, adaptive-policy,
downloader, partial-store, and loopback-gateway path. Preserve the current
playback backend and its public behavior except for focus, identity, and
observation mapping required by the fixes below.

## Start here

Read the repository rules and then trace the complete production path:

```text
Rust feed snapshot -> Dart roster/focus -> Rust adaptive plan
-> origin requests/partial store -> loopback gateway -> video_player
-> Dart playback observation -> Rust adaptive plan
```

Inspect commits `1ee153c` and `9f2fbb9`, including tests deleted by the former.
Use them as evidence, not as code to restore blindly.

Before production edits, add deterministic progressive-MP4 integration fixtures
that record focus, observations, policy allocations, origin requests, stored
ranges, gateway demand, first frame, stalls, and cancellation. The fixture must
exercise the Rust progressive path; the existing direct-HLS device test does
not prove that path.

## Fixes

### 1. Use one delivery identity everywhere

Confirmed defect: `FfiFeedPost.post_id` is the Rust delivery/cache identity,
while `event_id` is the social identity. Focus and gateway URLs use the former,
but `FeedCard` builds telemetry from `VideoPost.id` (the latter), so Rust rejects
real observations and schedules blindly.

Add failing tests where `post_id != event_id`, then:

- keep `VideoPost.id` for social actions;
- expose a strong delivery/playback ID from the media/cache scope;
- use that ID for focus, playback requests, gateway URLs, demand, and telemetry;
- centralize the mapping outside presentation code;
- record typed counters for accepted and rejected observations.

Primary references: `rust_feed_post_mapper.dart`, `feed_card.dart`,
`ffi_focus_item_media_mapper.dart`, `ffi_progressive_playback_gateway.dart`,
`ffi_playback_telemetry_port.dart`, and Rust playback admission state.

### 2. Repair roster reconciliation and tail loading

Confirmed defects:

- `FeedRoster.resynced()` discards unseen rows from a full Rust snapshot;
- passive refresh and append change the Dart roster without republishing focus;
- a duplicate or filtered-empty page stops backfill even when more pages exist.

Tests must cover unseen rows, revisions, active-row preservation, active-row
removal, same-index focus republishing, duplicate/empty pages followed by a
fresh page, exhaustion, and bounded no-progress retries.

Implement these semantics:

- preserve the active row and visible order;
- replace revisions in place and append unseen eligible rows after the tail;
- publish the full changed roster without recording a fake watch event;
- preserve `hasMore` and cursor progress even when the visible result is empty;
- continue through dry pages with an explicit, test-asserted per-drain cap;
  when the cap is reached and the cursor still advances, schedule the next
  attempt through a fake-clock-driven backoff instead of waiting for a swipe;
- trigger backfill before the user reaches the last item.

Primary references: `feed_roster.dart`, `feed_session.dart`, `feed_backfill.dart`,
`feed_cubit.dart`, `feed_cubit_loading.dart`, and `feed_viewer.dart`.

### 3. Reserve useful work for the immediate next video

Confirmed defect: both next-video paths subtract `planned_bytes(plan)` from the
speculative budget, but that total includes current-video bytes. On constrained
mobile conditions the subtraction reaches zero and the next video receives no
work.

Add policy tests with one connection, high RTT, low throughput, cold current,
and cold next. Then separate:

- current playback-safety work;
- immediate-next startability work;
- lower-priority speculative work.

Current emergency bytes stay first, but a servable next video must receive the
initialization and first useful media extent before deep current buffering. If
that is impossible, return explicit infeasibility evidence instead of silently
allocating zero. Here, servable means the next candidate has a live origin and
the policy can admit its bootstrap/startability bytes within current storage
and transfer constraints. Update `standards/VIDEO_QOE_TARGETS.md`: zero
immediate-next work is no longer valid for such a candidate.

Primary references: `rust/crates/engine/src/adaptive/policy.rs`,
`allocation.rs`, `resources.rs`, and the startup-breadth/playback-ahead tests
removed in `1ee153c`.

### 4. Retain demand from current and prepared-next players

Confirmed defect: delivery stores one `pending_demand`; a new signal overwrites
the old one, and reconciliation discards demand that is not for the current
item. A prepared next player can therefore wait for the gateway's 15-second
idle timeout after a swipe.

Add failing tests for simultaneous current/next demand, range advancement,
next-to-current promotion, fulfillment, stream cancellation/drop, roster
removal, and stale representation cleanup. Then replace the one-shot signal
with a small level-triggered lease protocol: each gateway response body owns a
consumer ID and emits `blocked`, `advanced`, and `released` state; the manager
keeps active leases keyed by consumer, canonical delivery/playback ID,
representation, and range. The body releases its lease on completion,
cancellation, or drop. No progress may depend on the idle timeout.

Primary references: `rust/crates/delivery/src/manager.rs`, `manager/wake.rs`,
`manager/reconcile.rs`, and the progressive gateway response stream.

### 5. Start useful bytes without waiting for a separate HEAD

Confirmed defect: cold candidates wait for total length/range evidence from a
HEAD probe before useful body work. A successful HEAD without `Accept-Ranges`
is treated as range-blind without trying a range GET.

Add failing tests for delayed/unsupported HEAD, absent `Accept-Ranges` with a
valid `206`, total from `Content-Range`, a range ignored with `200`, tail-moov
MP4, invalid MIME, mirror failover, and prefix reuse. Then add a typed policy
allocation that admits one bounded bootstrap range for focused current and
immediate next even while total length or range support is unknown. Only after
that admission may delivery issue `GET Range: bytes=0-N`; the response both
learns metadata and stores useful prefix bytes. A range-ignored `200` body must
remain capped. Preserve SSRF, redirect, content-type, size, and cancellation
protections. Every body request, including bootstrap, needs prior policy
admission.

Primary references: `rust/crates/delivery/src/probe/`, engine candidate
admission, chunk response handling, and progressive route snapshots.

### 6. Do not warm the first relay arrival as if it were canonical

Confirmed defect: discovery can arrive in a different order from the final
newest-first feed, yet startup pins the first arrival. Before explicit focus,
discovery may collect bounded metadata but must not claim displayed-current body
authority. The first canonical roster/focus grants that authority immediately.

Add failing tests for relay-order/canonical-order disagreement, focus
retargeting, and stale projection revisions, then change only the delivery
startup authority state. Primary references:
`rust/crates/delivery/src/manager/state.rs` and
`rust/crates/delivery/src/manager/state/probes.rs`.

### 7. Give focus to the visible feed surface only

Confirmed defect: home, discovery, and profile feeds share one static
`FfiFeedFocusPort`. Covering a feed only pauses its player; another route can
overwrite the one global Rust focus, and returning home refreshes through
`FeedViewer.stayedOn()`, which does not republish focus.

Add failing route and tab tests for home -> routed feed -> home, nested feeds,
tab changes, disposal, and late writes from an inactive feed. Then put a small
Dart focus arbiter in front of the existing FFI port. Each mounted feed owns a
surface lease; only the active visible lease may write. Activation or return
must republish that feed's complete roster and current index without recording
a fake watch event. Deactivation/disposal releases the lease, and stale writes
from inactive surfaces are ignored. Keep Rust's existing monotonic focus
generation; do not add a second player or per-route downloader.

Primary references: `app_controller_factory.dart`, `ffi_feed_focus_port.dart`,
`home_shell.dart`, `feed_route_scaffold.dart`, and `feed_viewer.dart`.

## Automated acceptance

Add one stable command for the progressive-path suite. It must prove:

- a mismatched social ID cannot disconnect accepted playback sensing;
- current work is first and immediate-next work is nonzero and useful;
- prepared-next demand survives until fulfilled or explicitly released;
- no blocked request waits for the 15-second timeout to make progress;
- completed origin byte ranges do not overlap;
- cancellation waste after ineligibility is at most `192 KiB`;
- reaching the tail triggers automatic roster and Rust-focus extension;
- before canonical roster/focus arrives, no discovery row holds
  displayed-current body authority;
- the cold fixture begins admitted useful body transfer in one origin round
  trip;
- after any route or tab transition, Rust focus matches the one visible feed's
  full roster and current item.

Run focused Flutter/Rust tests while iterating, then the repository-required
analyze, test, coverage, formatting, lint, and native test commands. Run the
progressive journey on the repository-owned Android AVD through an automated
target; creation, launch, test, and shutdown must require no manual interaction.
