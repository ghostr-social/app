# Rust Media Engine — Implementation Plan

Decision (2026-08-03): Rust owns all media **discovery, retrieval, and streaming**.
Dart is a pure client: UI, gestures, settings, identity/signing, and a
`video_player` pointed at localhost URLs. Scope agreements: social writes are
signed in Dart and broadcast by Rust; images stay on Flutter's image pipeline;
follow/mute lists are edited and published by Dart, consumed by Rust.

Migration is delivery-first (strangler): the app ships at every phase.

---

## 1. End-state architecture

```
┌────────────────────────── Dart / Flutter ──────────────────────────┐
│ UI, feed rendering, gestures, video_player, settings, keys/signing │
│      │ open_feed / update_focus     │ plays http://127.0.0.1/...   │
└──────┼──────────────────────────────┼──────────────────────────────┘
       ▼ narrow FFI (flutter_rust_bridge)          ▼ plain HTTP
┌────────────────────────────── Rust ────────────────────────────────┐
│ discovery  nostr_sdk — outbox NIP-65, search NIP-50, pagination,   │
│            trending, follows/mutes, dedup, imeta parsing           │
│ catalog    indexed posts + metadata (size/duration/sha256/host)    │
│ engine     tiers + scoring, inventory control loop, host stats,    │
│            HEAD probes, data-usage budgets                         │
│ transfer   ranged/resumable chunk downloader, partial-range store, │
│            streaming SHA-256                                       │
│ cache      ONE cache, full user budget, lease/evict                │
│ gateway    axum loopback — progressive MP4 from partial files +    │
│            existing HLS proxy                                      │
└────────────────────────────────────────────────────────────────────┘
```

Player urgency needs no FFI: the gateway sees the player's own Range
requests and promotes those bytes to the top tier directly.

## 2. FFI contract (freeze early, keep narrow)

Phase-1 surface (`rust/src/api/`, regenerated once, then stable):

| Call | Direction | Purpose |
|---|---|---|
| `ffi_start_engine(cache_dir, config)` | D→R | replaces `ffi_start_server`; returns endpoint |
| `ffi_set_delivery_config(data_usage, budget_bytes)` | D→R | live knob changes |
| `ffi_update_focus(feed_id, items, current_index, watch_ms)` | D→R | ordered window; each item: post id, urls, delivery kind, sha256?, size?, duration? |
| `ffi_playback_url(post_id) -> String` | D→R | localhost URL (progressive or HLS) |
| `ffi_delivery_events(sink)` | R→D stream | readiness per post, stats, errors |

Phase-2 additions: `ffi_open_feed(spec)`, `ffi_load_more(feed_id, older_than)`,
feed-update stream, `ffi_broadcast_event(signed_json)`. When `open_feed`
lands, `update_focus` items slim down to post ids (catalog already knows them).

## 3. Scheduler specification (the heuristics)

Unit of work: a **byte range of a video**, never a whole file. Per video:
probe → head chunk (first ~4 s + moov, what makes it "startable") → tail
chunks (~1 MB each).

Tiers first, score within tier:

- **T0 playback emergency** — playing video's buffer-ahead < 5 s (detected via
  gateway demand); all bandwidth here.
- **T1 current video tail** — watch time past ~3 s = commitment; finish it.
- **T2 startability** — head chunks for the upcoming window until the
  inventory target holds (default: 4 of next 6 startable).
- **T3 deepening** — tails, large files, slow hosts (comfort mode only).
- **T4 speculative** — beyond-window heads, scroll-back neighbours.

Within tier: `positionWeight(distance) × valuePerByte × hostSpeedFactor`.

Control loop modes with hysteresis: **hunger** (below target — cheapest
ETA-to-startable first, skip slow hosts, hard head-budget cap per video) and
**comfort** (deepen, admit big files and slow hosts). Scroll-past cancels a
video's in-flight chunks; fetched ranges are kept and resumable.

Host model: per-host EWMA of throughput, TTFB, failure rate; updated by every
probe and chunk; persisted as JSON in the cache dir. Drives ETA ranking and
best-URL choice among imeta fallbacks. `Accept-Ranges: none` marks a video
all-or-nothing (comfort-mode only unless imminent).

Metadata precedence: imeta `size`/`duration` (free) → HEAD probe → assumed
~2.5 Mbps refined from observed bytes.

Default parameters (one config struct, scaled by `DataUsageLevel`):

| Parameter | Default |
|---|---|
| Head budget | ~4 s, cap 3 MB |
| Chunk size | 1 MB |
| Startability target | 4 of next 6 |
| Deep target | current + next fully cached |
| Commitment threshold | 3 s watched |
| Emergency threshold | buffer-ahead < 5 s |
| Concurrent chunks (cons/bal/aggr) | 2 / 3 / 4 |
| Host EWMA half-life | ~10 transfers |

## 4. Phase 1 — Delivery engine (ships the fluid-feed win)

Dart-ndk still discovers; it feeds candidates to Rust via `ffi_update_focus`.
Rust becomes the only downloader and the only playback source.

Commit-sized steps, each test-first per AGENTS.md:

1. **imeta size/duration in Dart** — parse `size`/`duration` in
   `lib/features/video_catalog/data/nostr_video_media.dart`; carry on
   `VideoMediaSource`. (Parser tests; currently these fields are dropped.)
2. **FFI contract v1** — new `rust/src/api` module + FRB regen; Dart adapter
   behind a port (`lib/platform/media/`), fake for tests.
3. **Engine core (pure Rust, table-driven tests)** — new `rust/src/engine/`:
   `catalog.rs`, `focus.rs`, `tiers.rs`, `scoring.rs`,
   `inventory_controller.rs` (hunger/comfort + hysteresis), `chunk_plan.rs`,
   `host_stats.rs`, `budget.rs`. No IO in these modules; 100% coverage
   (repo gate for deterministic engine logic). Use `tokio::time::pause` for
   anything timed — no wall-clock flakiness.
4. **Partial-range store** — extend `rust/src/video/native_partial_store.rs`
   (today: cleanup bookkeeping only) into a sparse range store: on-disk data +
   range manifest, survives restart, resume, streaming SHA-256 finalization
   (rehash existing bytes on resume; verify against imeta `x` at completion).
5. **Ranged downloader + probes** — reqwest `Range` GET (accept 206), chunk
   grants from the engine; HEAD probe service on the existing SSRF-safe stack
   (`outbound_media_client.rs`, `public_dns_resolver.rs`).
6. **Progressive gateway serving** — implement the dormant `/video.mp4?id=`
   route in `rust/src/video/http_gateway.rs`: correct `Content-Length` from
   probe/imeta, Range responses served from the partial store, missing bytes
   stream as they arrive and promote to T0/T1. moov-at-end: head fetch sniffs
   moov; absent → also fetch tail ~256 KB before marking startable.
7. **Event-driven manager** — replace the 1 s poll loop in
   `rust/src/video/video_manager.rs` with reactions to focus updates, chunk
   completions, gateway demand, config changes.
8. **Dart playback via gateway** — new `ProxiedProgressiveVideoMediaSource` +
   port returning `VideoPlayerController.networkUrl` (no format hint);
   replaces the blocking full-file path in
   `lib/features/video_inventory/data/inventory_video_playback_surface.dart`.
   Add a `VideoPlayerValue` listener for stall/error UX + stats (none exists).
9. **Wire focus + config** — `FeedCubit.pageChanged`/load → `update_focus`;
   remove `FeedMediaPrefetcher` + `SmartVideoInventory` scheduling and the
   viewer-blind `prepare()` in
   `lib/features/video_inventory/data/inventory_remote_video_source.dart`.
   Full `VideoInventoryBudget` to Rust; delete the 50/50 split in
   `lib/features/video_inventory/domain/video_delivery_plan.dart`.
10. **Retire the freelancer + fallback shim** — stop index-driven
    auto-downloads; remove `FfiVideoRemoteSource` (accepted regression: no
    fallback feed when ndk fails; it couldn't paginate anyway).

Phase-1 exit: scroll a live feed with instant starts on prefetched heads,
downloads visibly reprioritizing on scroll; `flutter analyze`, `flutter test`,
`make test-coverage`, `cargo test` green; MANUAL_VERIFICATION.md entry.

## 5. Phase 2 — Discovery parity in Rust

1. Extend `rust/src/video/native_media_metadata.rs` with size/duration/dim/
   blurhash/thumb (becomes the only imeta parser).
2. Port to `nostr_sdk`: outbox directory (NIP-65), follows/mutes consumption,
   NIP-50 search, pagination windows, trending/query feeds (re-homes the Dart
   work from 9daf579 / 75a0dca / f372406).
3. Feed assembly in the catalog; `ffi_open_feed` / `ffi_load_more` + feed
   update stream.
4. **Unified control loop**: inventory hunger widens discovery; comfort
   quiets the radio. Relay budgets from `DataUsageLevel` move into the engine.
5. Writes: `ffi_broadcast_event(signed_json)` with outbox-aware relay
   selection. Keys never cross the FFI.
6. Feature flag in Dart: feed served by ndk (default) or Rust; shadow-compare
   both pipelines (log divergence) until parity holds.

## 6. Phase 3 — Cutover and deletion

- Flip the flag; remove ndk from the media path (Dart keeps only signing).
- Delete: Dart download stack (`smart_video_inventory.dart`,
  `file_video_cache_store.dart`, `video_cache_*.dart`,
  `http_video_file_downloader.dart`, transfer pool), Dart imeta parser,
  ingest-shaped `update_focus` payload (slims to ids), remaining
  `production_video_delivery*` wiring for the old paths.
- Final manual verification pass; update MANUAL_VERIFICATION.md.

## 7. Testing strategy (per AGENTS.md)

- Engine heuristics are pure functions → table-driven `cargo test`, 100%
  coverage; tuning happens in tests, not on-device.
- Gateway/downloader: Rust integration tests against a local fixture server
  (range semantics, 206/200, moov-at-end, resume-after-kill).
- Dart: port-level fakes for the FFI adapter; cubit tests for focus wiring;
  widget tests for playback states (happy/loading/empty/error, per required
  UI states). Existing specs to migrate, not delete:
  `test/video_catalog/feed_prefetch_follows_scroll_test.dart` pins the
  focus-follows-scroll behavior against the new port.
- Timing-sensitive tests use virtual time (`tokio::time::pause`,
  `fakeAsync`) — no deadline flakiness.

## 8. Risks and mitigations

- **ExoPlayer vs loopback progressive**: strict `Content-Length`/Range
  semantics required → probe before first serve; integration-test the exact
  header behavior; manual device pass early in phase 1 (step 6, not at the
  end).
- **FRB codegen churn**: freeze contract v1 (§2); one api module; additive
  changes only until phase 2.
- **Feed parity regressions**: shadow comparison behind the flag before any
  deletion (phase 2.6).
- **Background kill mid-download**: range manifest persists; resume on start.
- **Partial playback before full-hash verification**: integrity trade-off
  accepted; files are only marked complete/retained after matching imeta `x`.
- **Repo gates**: 200-line file limit and complexity caps shape the module
  split above; axum handlers stay thin over the pure engine.
