# WARP continuation handoff — 2026-09-05

This branch checkpoints unfinished work for resuming on another computer. The
real-relay playback objective is still open. Read `AGENTS.md`, the applicable
standards, and the complete `WARP-v3-final.md` before continuing. Preserve the
paper's knowledge, payload, activation, forecasting, watch-learning, dependency,
and budget contracts; reducing the design to current-video/next-video priority
does not satisfy the task.

Evidence artifacts are deliberately excluded from Git at the user's request.
Do not force-add `evidence/`, `.artifacts/`, logs, reports, screenshots, or APKs.
This note records implementation state and instructions, not the raw artifacts.

## Checkpoint boundaries

`16c8da3d` contains the real-phone test harness and the preceding fixes. Its
build-input fingerprint matched the last completed physical run before any
continuation changes were copied into this branch. The next commit preserves
38 additional Rust files from an isolated working copy. That continuation is
incomplete, has a known failing integration test, and has not passed the complete
Axiom gate or been run on the phone. Two temporary debug-output statements were
removed from its sustained-progress test when copying it into the repository.

Earlier local checks for the first checkpoint passed: Flutter analyze; 1,897
Flutter tests; coverage 98.80% with all 493 executable modules represented and
meeting the per-file gate; native focused suites; and complete-workspace Axiom
over 3,430 Rust files, including clippy and rustdoc. Later selection/disk and
corpus-diversity regressions also passed their focused checks. The physical
journey remained red. These are historical results, not validation of the
unfinished continuation commit. During checkpoint preparation,
`make video-live-android-contract-test` and `git diff --check` passed.

## Implemented before the continuation

- Real physical-phone journey uses configured Nostr relays, production feed
  mapping and signature verification, production delivery, and native playback.
  It measures motion after the first frame, backward navigation, rapid swipes,
  and paired direct-player controls. Direct controls have a dark labelled UI;
  the reported white video-only screen came from the earlier control harness.
- Delivery observation reads the app's existing subscription. Catalog scans
  no longer issue/refresh capabilities or evict the active lease, and unchanged
  failures are deduplicated.
- Public nonzero HTTP(S) media ports pass the production destination policy.
  Existing private-address, DNS, credential and redirect restrictions remain.
- Origin bitrate units no longer multiply an already-bit-based estimate by
  eight. Repeated callbacks within one stall episode cause one adaptive setback.
- WARP frontier dominance preserves concrete byte effects, source identity,
  dependencies, request/authority contracts and information effects. Similar
  scalar forecasts alone do not make unrelated requests interchangeable.
- Anonymous `Vary` selections are hashed from the final-hop selecting request
  headers and carried through source/HTTP generations, catalog observations,
  disk state and cancelled-whole-response resumption. Eligible variants are
  partitioned, never public. Credentials, wildcard/malformed Vary, private or
  no-store responses, and Set-Cookie remain restricted. A real public-origin
  regression is available through `make video-live-origin-test`.

The latest physical journey still produced frames for only 8 of 20 fresh videos
across 12 hosts; 12 produced no first frame. One successful startup subsequently
froze. All tested direct controls rendered a frame, although one Pexels control
showed no meaningful motion. The Vary fix improved the paired Libernet failures,
but weak-validator large objects and other hosts still fail. Do not equate
passing host tests or a favorable single-host run with successful playback.

## Unfinished continuous-response work now in this branch

Large weak-validator media must use one continuous response. The existing
planner reserves the entire file against a short rate burst, so a 16 MB object
cannot start under a roughly 1.6 MB burst. The continuation introduces renewable
256 KiB network windows while retaining full storage/coverage reservation and
the broker's cumulative body envelope. Windows consume the existing planner
token bucket; cancellation and attempt identity gate renewal.

Relevant implementation areas:

- Engine: `adaptive/retrieval.rs`, WARP generation allocation and planner
  `network_window`, plus both recorded/executed resource-accounting boundaries.
- Delivery: manager `body_renewal`, inflight/worker renewal checks, transfer
  traffic renewal, and `chunk/stream/whole.rs`.
- Traffic: local admission waits split measurement windows without inventing
  another TTFB or blaming the origin for local token waits.

New tests include `warp_continuous_body_budget_test`, planner network-window
tests, `traffic_body_renewal_pause_test`, and two real-TCP production-manager
integration tests using `continuous_body_fixture`.

`continuous_weak_validator_body_test` still fails: the initial readable prefix
does not appear within ten seconds. `continuous_body_renewal_progress_test`
passed after about 12.6 seconds because the entire response reached EOF. That
pass proves neither early prefix visibility nor fluid playback. Strengthen its
assertion to distinguish active streaming from completion before accepting it.
The traffic-pause regression passed. An earlier complete engine run had 534
passes and one old whole-budget expectation failure; that expectation was then
updated for renewable windows and has not been rerun. The complete continuation
still needs formatting, Axiom, accounting review, and broader regression checks.

## Immediate next step: action-scoped prefix visibility

Admission alone is insufficient. Partial-store deliberately stages
action-scoped whole responses until EOF, withholding them from the gateway.
No partial-store changes for exposing this prefix have been implemented yet.

Start with failing store tests for prefix visibility during an active response,
cancellation revocation, no completion before EOF, restart non-reuse, authority
identity, replacement preservation, and storage accounting. Inspect:

- `partial_range_store/single_response/opening.rs`: action-scoped storage is
  forced to the staged path.
- `single_response/staged.rs`: sequential writes and progress accounting do
  not publish a readable prefix; the current writer fsyncs each chunk.
- `single_response/session.rs`: a session response currently means complete;
  registration, checksums and notification happen only after EOF.
- `queries.rs`, `queries/session.rs`, `queries/media_snapshot.rs` and
  `finalize/session.rs`: completion is currently assumed for session responses.
- `single_response/lifecycle.rs` and `finish.rs`: storage accounting and abort
  cleanup must avoid double-counting or retaining a revoked visible prefix.
- Gateway `runtime/progressive.rs`: it needs a total and current representation
  binding; do not weaken HTTP generation authority to make the URL appear.

A possible design is explicitly typed active-prefix versus completed-session
visibility, bound to the response action. Publish only coherent bytes from that
one active response, with local read checksums. Do not permit cross-response
assembly, restart reuse, premature completion or canonical replacement. Preserve
the previous canonical object during a replacement response. Cancellation must
revoke prefix visibility immediately. Content revisions and byte accounting
must remain coherent through first visibility, later chunks, EOF and abort.
This is a proposal to test, not an implemented contract. Routing everything into
the transient whole-object memory cache fails for large objects and is not a
streaming solution.

Review renewal accounting as well: reply cancellation after a granted window,
unused-credit settlement, response identity, local-wait measurement, and an
underlying HTTP chunk larger than the currently granted read window. The current
loop splits stored output across credits, but reqwest may already have returned
a larger chunk into memory. Preserve cumulative broker limits.

## Other unresolved investigations

- Unknown whole-body header bounds can repeat a 1 MiB capped request when
  advertised and observed lengths conflict. `WholeBodyBoundDiscovered` is not
  currently recorded as whole-body exhaustion. Preserve generation identity
  when learning a usable size from response headers.
- Forecasts query current origin occupancy plus one even when a hard per-origin
  cap permits only one request. This can select an unlearned concurrency bucket.
  Model admissible service and queue wait accurately; do not simply discard it.
- MP4 `media_timeline/classic.rs` rejects any `edts` box. Correct edit-list
  support needs movie/track time conversion, offsets and decoder dependencies;
  ignoring edit lists would produce false readiness claims.
- Initial feed waits on remote social reads with a 15-second deadline. Preserve
  the known local mute/block floor and account authority when changing startup.
- Investigate HLS bootstrap/cache reuse and activation; duplicate-looking stages
  do not yet prove identical duplicate HTTP requests.
- Evaluation byte accounting misses HLS broker traffic, while watched bytes can
  be estimates. Do not claim bandwidth savings from the existing counters.

## Resume and verify

Build using the repository's normal Rust/Flutter setup. On the previous host,
two Cargo jobs and a fast local temporary directory avoided resource pressure;
the old absolute `/private/tmp` and external-SSD paths are not prerequisites.

```sh
cd rust
cargo test -p ghostr-engine --lib
cargo test -p ghostr-delivery --all-features --lib
cargo test -p ghostr-delivery --all-features --test continuous_weak_validator_body_test
cargo test -p ghostr-delivery --all-features --test continuous_body_renewal_progress_test
cd ..
make video-live-android-contract-test
make video-live-origin-test
```

After fixing the remaining behavior, run Flutter analyze/tests, the repository
coverage targets and `make axiom` before claiming completion, then repeat the
physical journey with fresh public videos and paired regression cases:

```sh
LIVE_COLD_CACHE=true LIVE_COLD_CACHE_KEY=resume-session-a \
  make video-live-android-evidence ANDROID_PHYSICAL_SERIAL=YOUR_PHYSICAL_SERIAL
```

The previous phone was a Xiaomi M2012K11AG running Android 13. Its signed-in
account used Aggressive data usage and a 4 GB inventory; the VPN-active network
was classified Constrained. The harness backs up/restores the installed APK
using `adb install -r`; never clear account data or uninstall it. The restored
normal APK predates the latest fixes. A new isolated cache key starts a new model
and payload directory; reusing a key preserves learning, so describe it accurately.
Pinned replay uses `LIVE_VIDEO_EVENT_IDS` with comma-separated public event IDs.

Fresh-corpus exclusion currently reads local `evidence/warp/**/markers.log` via
`tool/live_video_prior_corpus.py`. Those files intentionally do not travel in Git.
A fresh checkout therefore lacks the previous machine's exclusion history until
it is supplied separately. Preserve the maximum-five-per-host rule and collect
new reports locally; do not add measurement artifacts to the branch.
