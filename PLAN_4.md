# PLAN 4 — Integrate Dashlet into the production retrieval scheduler

## Goal

Use Dashlet for production retrieval decisions while Rust continues to own
origin requests, exact range admission, cancellation, partial storage, and the
loopback gateway. Flutter's existing `video_player` stack remains the player.

This task starts from the repository as it exists. It must not assume another
branch, crate, report, or generated artifact is available.

## Establish the Dashlet reference first

- Repository: <https://github.com/PrincetonUniversity/Dashlet>
- Pinned commit: `33cf3688dcfda57915a3086f075eb76b15c3da29`
- Paper: <https://www.usenix.org/conference/nsdi23/presentation/li-zhuqi>
- First Python entry point: `abr-server/abrAlgorithmCollection_dashlet.py`

If `rust/crates/dashlet-scheduler/` is absent, create it in this task. Before
writing Rust scheduler code:

1. Fetch and verify the pinned commit, then trace the real Python driver,
   imports, and call path—not only the named file.
2. Record functions, state, inputs/outputs, units, indexing, constants,
   rounding, tie-breaking, swipe/watch distributions, bandwidth estimation,
   chunk ranking, and bitrate selection in `PYTHON_REFERENCE_MAP.md`.
3. Instrument complete multi-decision runs and commit canonical JSON fixtures
   containing inputs, intermediate scores/state, and final chunk/bitrate order.
4. Run the real Python functions from a committed interpreter/dependency lock;
   pin every random seed. Normal Rust tests use the fixtures offline.
5. Write failing Rust parity tests before the port. Python behavior is the
   compatibility oracle; the paper explains and cross-checks it. Document every
   difference with a reproducing fixture.

The crate stays pure and deterministic: explicit state in, decision plus next
state out. It performs no network, storage, gateway, Flutter, or FFI work.

## Prove the existing production path before changing policy

Add one deterministic progressive journey that crosses the real feed focus,
playback observation, Rust manager, origin downloader, partial store, loopback
gateway, and `video_player` adapter. It must prove:

- focus, gateway demand, and observations use the same delivery ID;
- the visible feed owns Rust focus and publishes its complete roster;
- cold startup begins a policy-admitted useful body request without a serial
  HEAD dependency;
- current and prepared-next gateway demand both survive until fulfilled or
  released;
- a servable immediate next receives startability work;
- reaching the feed tail extends both the UI roster and Rust focus without a
  new swipe or page change.

If the current repository fails one of these contracts, add the failing
regression at that boundary and make the smallest repair in this task before
using the path as the comparison baseline. Do not credit Dashlet for hiding a
broken observation or delivery path. Keep SSRF and redirect protections intact.

## First write failing Dashlet integration contracts

Freeze one production planning input and prove before changing actuation:

- baseline and Dashlet evaluate the identical immutable input;
- shadow evaluation starts no transfer, cancels nothing, and changes no store,
  history, focus, or scheduler state;
- every Dashlet result is validated before it becomes an `AllocationPlan`;
- exact blocked demand and current emergency work cannot be displaced;
- a servable immediate next cannot silently receive zero useful work;
- every body range still passes origin, storage, representation, commitment,
  and duplicate-byte admission;
- unsupported or unrepresentable input returns a typed fallback reason.

Use fixed-seed progressive fixtures for low/high bandwidth, high RTT, cold
start, rapid forward swipes, backward navigation, storage pressure, unknown and
complete-file layouts, equal-score ties, and multiple verified renditions.

## Production design

### 1. Add one strategy boundary

For fixed-rendition scheduling, extract the pure seam around
`AdaptivePlayabilityPolicy.plan(&PlayabilitySnapshot)`. Keep baseline,
Dashlet-shadow, and Dashlet strategies. All return the existing
`AllocationPlan`; delivery code must not receive Dashlet models.

Select the configured strategy once when the delivery runtime starts. A typed
per-pass baseline fallback is part of the Dashlet strategy; it is not a live
runtime mode switch.

### 2. Preserve a minimal safety envelope without neutering Dashlet

Outside Dashlet, reserve only:

1. the exact byte range currently blocking a gateway consumer;
2. the smallest current-video extent needed to cover the predicted time until
   the next admitted extent can arrive when playback is endangered;
3. existing in-flight commitments that are still valid under current
   cancellation, representation, and focus rules.

Do not reserve the full current target buffer or the complete immediate-next
startup set. Dashlet must rank the remaining current extension, next-video
startability units, and farther candidates; that current-versus-next choice is
the algorithm being evaluated.

Validate the translated plan before actuation. When a servable next exists and
budget remains after the emergency floor, the admitted prefix must contain
positive next-startability work before any farther candidate or current bytes
beyond the normal target. A violation produces a typed baseline fallback in
production and fails every deterministic integration fixture.

Origin choice, exact byte admission, storage limits, eviction, commitments,
duplicate suppression, and cancellation remain owned by the existing engine.
Dashlet never opens a URL or writes the store.

### 3. Integrate scheduling at one fixed rendition first

Map only timeline-backed playable extents with explicit time, bytes,
dependencies, and the currently verified representation identity. Do not
pretend arbitrary byte slices are aligned DASH/HLS segments. If the current or
required next item cannot be represented faithfully, use one typed baseline
fallback for the whole planning pass rather than mixing incompatible plans.

At this stage Dashlet contributes chunk ordering; its bitrate output is excluded
by the adapter type. Single-rendition media keeps its existing bitrate.

If the crate returns one decision per call, build the bounded `AllocationPlan`
by advancing a copy of its explicit state until the available lanes/budgets are
filled. Shadow evaluation never commits that state. Dashlet mode commits only
steps whose exact ranges were admitted for actuation; rejected or fallback work
must not advance scheduler state.

### 4. Add ABR only through a real rendition seam

Before enabling Dashlet bitrate selection, add a failing Python-parity fixture
with at least two verified renditions. Trace whether the Python path couples
bitrate and chunk order:

- if independent, add a narrow pure rendition-selection input before the
  existing representation binding;
- if coupled, add one pure retrieval-policy input/output containing the verified
  ladder and a representation identity on every selected unit.

Do not split a coupled Python decision merely to fit current types. Runtime
ladders must contain real sizes, timelines, and identities; deterministic test
fixtures may model multiple real renditions. Never fabricate a ladder,
transcode media, or add another downloader.

### 5. Supply the distribution the Python code actually consumes

Derive bucket definitions and transition semantics from the executed Python
path. Record dwell time and forward/backward navigation from a monotonic Rust
clock. Use the artifact's default prior when present; otherwise use one
documented normalized cold-start prior and replace it with observations.
`watch_ms = 0` is absent evidence, never a measured watch duration.

Test every bucket boundary, cold start, natural completion, forward/backward
swipes, session reset, and repeated identical traces.

### 6. Shadow both strategies on the same input

Compute and record baseline and Dashlet results before either can mutate
delivery state; shadow mode actuates baseline only. Record ordered units,
representation choice when applicable, rebuffer score, validation result, and
typed fallback reason.

Add one stable `make dashlet-integration-gate` command. With identical fixtures,
impairment profiles, and fixed seeds for both strategies, compare p95:

- cold-start latency;
- time from accepted focus to the immediate next holding initialization plus
  its first playable extent;
- focus-switch latency;
- rebuffer ratio;
- cancellation waste;
- duplicate completed origin bytes and admission violations.

### 7. Activate automatically only after the gate passes

The gate must enforce every existing absolute QoE/safety target, no regression
greater than `5%` in cold-start latency, focus-switch latency, or rebuffer ratio,
and at least `5%` improvement in next-startability time, focus-switch latency,
or rebuffer ratio. The task remains incomplete until the gate is green, the
production default is Dashlet, and a composition test asserts that default.
Keep baseline compiled and selectable for typed fallback and rollback.

Unsupported input, a typed crate error, or NaN/invalid ranges caught by result
validation before actuation fall back for that pass and record the reason. A
Rust panic, missing admission, duplicate origin request, or invariant violation
in the fixed fixtures is a defect, not an accepted fallback; do not add
`catch_unwind` as a substitute for correctness.

## Boundaries and completion

- Do not edit upload, publishing, transcoding, HLS packaging, GStreamer, or
  native-player code.
- Do not call Python at runtime or require network access for normal tests.
- Do not bypass the SSRF-safe client, range admission, partial store, gateway,
  or cancellation rules.
- Do not remove the baseline strategy.

Completion requires the Python/Rust parity suite, production-path journey,
adapter and result-validation tests, shadow side-effect tests,
`make dashlet-integration-gate`, full Rust tests, native coverage/dead-code
checks, progressive browser acceptance, and the automated Android AVD journey
to pass. The final report lists traced Python functions, mapping/fallback
reasons, exact paired metrics, tests changed, and command results.
