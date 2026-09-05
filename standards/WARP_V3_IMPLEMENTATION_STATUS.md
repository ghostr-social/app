# WARP v3 implementation audit

Authority: [WARP-v3-final.md](../WARP-v3-final.md), revision 3.0, 4 September 2026.
The complete physical matrix, normal ARM64 app launch, Flutter analysis,
Flutter coverage, native tests, and full-workspace Axiom gates pass.

## Scope and changes

The enabled production profile is CORE/3. The specification explicitly gates WRM/1,
VPK/1, PUB/1, JIT/1, and LOOKAHEAD/1 separately. Those profiles are disabled and
are not claimed here. Ordinary media does not require sidecars. The referenced
implementation pack has not been supplied or found in the workspace.

- Deterministic current/next scheduling is the default. Digital-twin proposals,
  adaptive prices, generated hedges, and transport-based feed reordering require
  explicit experimental configuration. Semantic order survives delivery failures.
- Unopened range promotions do not suppress ordinary whole-object fallback.
  Useful current/next transfers survive replanning until actual terminal evidence.
  Cancelled exact whole responses retain independently resumable strong-validator
  prefixes, with retained-byte accounting and reduced replay on real swipes.
- HTTP generations, range geometry, action leases, output hashes, and mirror
  rejection remain separate authorities. A failed mirror no longer poisons every
  claim sharing an advertised digest. Parser facts belong to the observed source.
- MP4 indexes preserve exact composition timing, sync dependencies, required-track
  coverage, and sample ranges. Unsupported edits and description switches use
  ordinary acquisition. Checksummed, bounded local indexes survive payload eviction;
  fresh source evidence is required before reuse. Indexes do not certify readiness.
- Continuation planning uses dependency-complete arrivals and pre-arrival deficit,
  recalculates from position and actual buffer, preserves uncapped requirements,
  and accounts for commanded pause/rate. Origin estimates are constrained by the
  shared path and conservative allocation to two protected items.
- Metadata exploration is bounded to eight ahead, encoded preparation to the next
  two items, active requests to two globally and one per origin, and native players
  to active plus immediate next. Memory pressure reduces players to one.
- Cache recovery protects a cold current video before future whole downloads,
  including the interval before decoder byte-demand telemetry exists. Whole-file
  fallback makes room before admission; ordinary range bootstrap does not reserve
  an optional whole response. Cold entries outside the feed working set can be
  reclaimed while active read leases and working media remain protected.
- Late teardown proof can restore quarantined capacity; missing proof cannot.
  Empty focus cancels HEAD workers while counting them until acknowledgement.
  Removed previous/deep player hosts, obsolete retention tests, unused claim
  bookkeeping, and redundant helpers/assertion boilerplate.
- Cumulative Internet reservations are durable before dispatch and shared by media
  consumers. Cancellation/crash retains conservative charges; time, connectivity,
  and restart do not refill them. The product default is Unlimited. Finite limits
  are native gateway configuration, not a new user settings flow.
- Private, no-store, signed/contextual, and validatorless responses without public
  cache policy use bounded volatile playback buffers. Hash verification never promotes them to disk.
  Logout revokes progressive URLs and HLS sessions, cancels media work, and removes
  private buffers while retaining public media for fresh authorization.

## Verification

| Check | Current result |
|---|---|
| `flutter analyze --no-pub` | Pass; no issues |
| `make test-coverage` | Pass; 1,890 tests |
| `make coverage-summary` | Pass; 98.80% (14,741/14,920), all 493 modules and executable-source representation |
| `make native-test` | Pass; 2,375 tests across 450 target results |
| `make axiom` | Pass; all 3,409 Rust files, Clippy, rustdoc, and semantic policies |
| Complete physical matrix, offline restart, HOME/foreground | Pass; 26 matrix cases + two offline phases + one lifecycle case |
| Normal ARM64 app | Build/ABI check, installation, cold launch, and visible welcome screen pass |
| `make site-check` | Pass; preserved landing-page deployment dry run |
| `make warp-evidence-contract-test` | Pass |

The [acceptance map](WARP_V3_ACCEPTANCE.md) links the concrete automated entry
points. The [release manifest](WARP_V3_RELEASE_MANIFEST.md) pins dependencies,
resource limits, supported profiles, and platform scope.

New and strengthened regressions cover:

- Current-video reacquisition after eviction, feasible whole-file storage admission,
  partial range reservations, and reclamation outside the feed working set.
- HTTP generation and mirror isolation, malformed and oversized responses, transient
  private playback, cumulative allowance persistence, and logout revocation.
- Composition timing, selected-track dependency closure, retained-index generation
  fences, shared-path service deficits, pauses, and playback-rate changes.
- Player teardown capacity, stale callback rejection, immediate-next preparation,
  semantic navigation order, and actual native request occupancy.

Physical checks use the connected Xiaomi M2012K11AG, Android 13, serial 22e0d933.
They assert native decoded-frame telemetry and advancing playback positions,
resource bounds, payload accounting, and generation-specific playback authority.
The cache-pressure case actually evicts and reacquires a video. The long session
performs 32 handoffs across 24 posts. HLS checks separate structural readiness,
decoded authority, and lease cleanup. Test setup waits for rendered UI; malformed
range rejection is isolated from a later valid whole-file fallback. Cache evidence
handles a manifest removed by concurrent eviction while preserving other read and
parse failures; dedicated regression tests cover both outcomes. The fixture relay
queues each finite response synchronously so shutdown cannot interrupt an attached
stream or leave an end-of-response write pending. A real-WebSocket regression
sends another request when the first event arrives and starts relay shutdown;
the explicit shutdown state rejects that queued request and late connections.

Long-session replay is reported separately from cancellation overrun. The
controlled cancellation still enforces its 192 KiB overrun target. Session-wide
replay can include safe reacquisition after a whole response is cancelled before
completion, even if the origin sent the advertised byte count. The
[cancellation-before-EOF test](../rust/crates/partial-store/tests/partial_range_cancel_before_eof_test.rs)
confirms that such bytes cannot become reusable complete media. Completed-response
duplicates remain forbidden, and all origin extents and resource limits remain
asserted.

Fixture regressions:
[warp_cache_manifest_eviction_race_test.dart](../test/media/warp_cache_manifest_eviction_race_test.dart)
and [warp_feed_relay_shutdown_test.dart](../test/media/warp_feed_relay_shutdown_test.dart).

The native orphan-cache regression also owns its test runtime explicitly and
removes its temporary directory only after shutdown joins pending filesystem
work. The full suite exposed the earlier cleanup race as `DirectoryNotEmpty`;
all playback and capacity assertions remain intact.

Final physical evidence is recorded in
[evidence/warp/20260905T104209Z-ec364531d3-22e0d933-physical-matrix](../evidence/warp/20260905T104209Z-ec364531d3-22e0d933-physical-matrix).
Completed host logs are collected alongside the device results. Earlier runs
remain diagnostic evidence for their recorded source states.

## Source and validation scope

The changes are integrated into local `main`, preserving the landing page and
its deployment workflow. The [host verification record](../evidence/warp/20260905T104209Z-ec364531d3-22e0d933-physical-matrix/host-verification.txt)
records the final commands, counts, and environments.
The phone run changed only a host-only Rust test assertion; the source-change
attestation reconstructs the exact original fingerprint and confirms unchanged
production and device-test sources. The normal app was visually checked at the
welcome screen; no account was configured. No iOS or browser-only physical
validation, optional-extension rollout, or canary performance claim is made.
