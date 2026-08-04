# Handoff — `rust-media-engine`

Goal: **every Nostr request is issued by Rust.** Dart keeps the UI, the
player, and the keys — it signs events and never opens a relay socket.

Six commits, all green (`cargo test`, `flutter analyze`, 819 Dart tests).
Background and rationale: [RUST_MEDIA_ENGINE_PLAN.md](./RUST_MEDIA_ENGINE_PLAN.md).
Four device passes: [MANUAL_VERIFICATION.md](./MANUAL_VERIFICATION.md).

## What is already done

**Delivery (done, validated on device).** Rust owns video bytes end to end:
a scheduling engine (`rust/src/engine/`) with tiers, scoring, a
seconds-of-inventory control loop, and per-host EWMA stats; a ranged,
resumable chunk downloader over a sparse partial-range store; and an axum
loopback gateway that streams progressive MP4 out of partially-downloaded
files. The player points at `http://127.0.0.1:PORT/video.mp4?id=…`, and its
own Range requests are the top-priority demand signal. Proven on device:
a range starting inside a hole returned bytes whose sha256 matched the
completed file. The whole Dart download and network stack is deleted.

**Discovery (built, not yet serving).** `rust/src/discovery/` has event
parsing, NIP-65 outbox routing, follows/mutes, NIP-50 search, pagination,
feed assembly with profile enrichment, trending, and a scheduler whose
query pressure follows the delivery engine's inventory mode. FFI v2
(`rust/src/api/`) exposes `open_feed`, `feed_updates`, `load_more`,
`close_feed`, `broadcast_event`, plus the delivery calls.

**Dart can serve feeds from Rust but does not.**
`lib/app/feed_pipeline_flag.dart` defaults to `FeedPipelineMode.ndk`.
`shadow` runs both pipelines and logs divergence (`ghostr.feedparity`)
without touching what the user sees.

## Blocker 1 — feed parity (holds the cutover)

Across four device passes the Rust feed returns **fewer rows** than ndk on
both main feed and search. Ordering always agrees (0 order mismatches in
202 records); the disagreement is membership only.

Ruled out already, with evidence: adapter sampling mid-fan-out (Rust
snapshots are settled pages, one plan → one revision); wire limits (both
sides use 80 narrow / 200 wide); relay coverage alone (NIP-65 routing is
now wired and it did not move the gap); a cold event pool (the client's
`MemoryDatabase` had `events: false` — now enabled and unioned; it
demonstrably serves rows that were previously missing, but `missing` did
not shrink).

**Do this first, before writing another fix.** `FeedParityDivergence`
(`lib/features/video_catalog/data/feed_parity_divergence.dart`) caps each
list at 5 ids and reports no counts, so every record so far reads
`missing=[5 ids]` — a lower bound. `|missing|` is pinned at exactly that
cap and flat across 90 minutes of session age, which says Rust's page is
systematically smaller, but not by how much. Make the reporter emit page
sizes and totals, run one shadow pass, and you will know whether this is
20-vs-25 rows (cut over now) or 20-vs-200 (a real hole). Fixing blind has
cost four passes.

## Blocker 2 — the rest of Nostr still runs on ndk

`lib/core/nostr/nostr_event_client.dart` is the seam, and it is small:

```dart
abstract interface class NostrEventClient {
  Future<List<NostrEventRecord>> query(NostrEventQuery query);
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> queries);
  Future<NostrEventId> publish(...);
}
```

Engagement (`lib/features/engagement/data/`), comments
(`lib/features/comments/data/`), publishing, and profile search all sit
behind it. **Implement this port over Rust FFI and every remaining feature
moves at once**, without touching feature logic:

- add a generic `ffi_query_events(filters) -> events` (+ a batch form) to
  `rust/src/api/`, routed through the existing `DiscoveryScheduler` so it
  shares the outbox directory, the event pool, and the data-usage budget;
- map `publish` onto the existing `ffi_broadcast_event` — Dart still signs,
  keys never cross the boundary;
- swap the adapter at the composition root, keep the ndk one until it is
  proven, then delete it.

Note Rust has **no reaction or comment aggregation**. Counting kind-7
reactions and threading comments has to exist in Rust before ndk goes, or
every like and comment count silently reads zero. Trending needs no work:
`RecentVideosTrendingHashtags` derives from feed results, so it follows the
feed automatically.

Once the port is Rust-backed and parity holds: flip the flag, delete
`ndk_video_remote_source.dart`, the Dart imeta parser
(`nostr_video_media.dart`, `nostr_video_event_mapper.dart`), and the query
builders in `lib/platform/nostr/`. Keep session/signing and Blossom upload
(an HTTP upload, not a relay call).

## Live defects worth fixing

- **57 `Video player initialization failed` in the last pass** (was 7): 39
  `UnrecognizedInputFormatException`, 15 `InvalidResponseCodeException`
  (404/503 from the loopback gateway). Something is being admitted and
  served that is not playable — one case was a still image
  (`image.nostr.build/…`) in a video feed. Start at gateway 404/503 paths
  and at admission in `rust/src/discovery/event_parsing.rs`.
- **`cannot write into a finalized video`** warnings (10 in one session):
  a transfer still targets a post after finalization.
- Rust returns `Err` on a network failure instead of falling back to stored
  pool rows — a warm pool could answer offline.

## Working rules

- `AGENTS.md` is binding: test-first; production files ≤200 lines; test
  files ≤100 lines, one behavior each; functions ≤20 logical lines and ≤4
  params. Several files sit at the limit — split, don't overflow.
- FFI changes go through `make gen`; never hand-edit `lib/src/rust/**`.
- Device testing is the only way to catch this class of bug — the last four
  passes found a retry storm, a disk-filling cache, and two startup paths
  that wiped the store on every launch, none of which any unit test saw.
  `MANUAL_VERIFICATION.md` documents the exact working procedure
  (x86_64 ABI, profile APK, VM-service log capture — `dart:developer.log`
  does not reach logcat). Reuse it; rediscovering it is expensive.
- The ndk baseline is not a gold standard: it logged 287 relay connection
  failures and 40 unhandled exceptions in one session, and degrades to a
  blank feed after 8–30 minutes. If parity turns out to be close but not
  exact, "match ndk" may be the wrong bar — that is a product call.
