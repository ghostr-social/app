# Video QoE Targets

These are release gates for the adaptive video-delivery path. They are SLOs,
not a claim that playback can be literally uninterruptible on every network or
device.

## Automated acceptance budgets

| Metric | Target | Measurement |
| --- | ---: | --- |
| Adaptive plan evidence | Every row | Ordered production plan revisions with exact ranges, positive playable gain and delivery cost, utility, authority, commitment, and reason. |
| Healthy adaptive expansion | `>= 2` distinct admitted posts | On a fresh cache with focus held, measured healthy capacity admits more than the current post; no exact frontier size is prescribed. |
| Origin admission | `100%` | Every origin body request exactly matches an allocation published before that request began. |
| Network response | Changed plan evidence | A bandwidth or loss change either contracts breadth or changes delivery-cost evidence for preserved exact work before the next focus change. |
| Cold startup latency | `<= 2,000 ms` | User focus intent to the first `playing` observation for that video. |
| Focus-switch latency | `<= 1,500 ms` | Each non-superseded focus intent to `playing` for its destination. |
| Rebuffer ratio | `<= 1%` | Time in `buffering` or network-stalled phases after first play, divided by observed playback time. |
| Cancellation waste | `<= 192 KiB` | Origin body bytes sent after focus left a request that was then canceled. |
| Ahead prefetch | `<= 3 MiB` | Maximum stored sibling bytes gained while the current video is active and incomplete. Zero immediate-next work is valid only when the plan records `NextReserveEvidence::Infeasible`; constrained capacity alone cannot silently consume a servable candidate's reserve. |
| Duplicate completed origin bytes | `0 B` | Successful completed body-range overlap previously fetched from the same exact source. |
| Protected transition latency | `<= 500 ms` | Tagged current-to-next transitions from click intent to the first `playing` observation. |
| Held-response recovery | `<= 2,000 ms` | Device fixture response release to resumed `playing`. |
| Manifest-retry startup | `<= 4,000 ms` | Focus through one same-URL manifest HTTP 503 to first `playing`. |

The browser contract is defined in `tool/video_user_e2e/qoe_targets.mjs`.
Device constants live in
`integration_test/support/device_qoe_targets.dart`. A change to either budget
requires a failing contract test and an explanation backed by retained traces.
The Android gesture injector targets 150 ms to leave its coarse live-device
pump below the unchanged 300 ms observed-cadence gate; this is stress
headroom, not a relaxed or redefined QoE target.

The browser matrix admits eight ordered videos. The `adaptive_plans` row starts
with a fresh private cache, explicitly focuses the first video, and holds that
focus for four seconds. It requires healthy capacity to expand policy-selected
coverage beyond the current post, but intentionally does not encode a fixed
frontier or per-post byte threshold.

Every impairment row runs in its own process with another fresh private cache.
The runner retains the bounded production plan history once at the end of the
journey and keeps sampled telemetry compact. Exact shared-origin request and
chunk chronology is authoritative: every body range needs prior exact policy
admission, successful overlap is forbidden, and network replanning may not
restart origin bytes already paid for during the impaired interval.

Its progressive fixture is a valid six-second H.264 MP4 whose timed samples
span `285,652` bytes and several `64 KiB` transfers. The origin and registration
advertise that exact size and duration; no virtual zero tail is counted as
media. Eight fixtures also exceed the `2 MiB` pressure budget.

## Deterministic impairment matrix

The browser acceptance suite covers:

- held-focus adaptive expansion on a fresh cache with a variable frontier;
- one shared link stepping from 2.5 Mbps to 700 Kbps, then recovering to 2.5 Mbps;
- 6,000 bps deterministic packet-loss injection plus failure of the first two
  `v2` body attempts after 64 KiB;
- 450 ms RTT at 2.5 Mbps;
- four focus changes at 200 ms intervals;
- a 2 MiB storage ceiling followed by an explicit capacity release;
- an HTTP 503 from the primary progressive source with a healthy mirror.
- protected transitions at 2.5 Mbps, 100 ms RTT, and one host connection.

The packet-loss journey observes its twice-impaired protected video for at
least `2.5 s`; other destinations use the `0.75 s` transition window. This
prevents a second partial response from passing on a briefly playable prefix
before a long retry cooldown can surface as rebuffering.

A scenario passes only when its retained trace proves that the fault was
material. Packet loss must inject two failures into protected video `v2`, with
one during its clicked interval. Source failure must record the selected
primary's `503` before a completed mirror body. Storage pressure must plateau
within one `64 KiB` transfer chunk of `2 MiB` and grow after release. The
bandwidth drop must be visible while an incomplete transfer is active, followed
by a trace-verified recovery profile. Bandwidth and packet-loss rows also need
a production plan response while exact useful origin work is preserved. High
RTT must be sampled during active, incomplete delivery at exactly 2.5 Mbps,
450 ms, and three host connections. Protected transitions likewise require a
sampled active transfer at exactly 2.5 Mbps, 100 ms, and one host connection;
bootstrap control receipts alone do not satisfy either activation guard.

The browser run uses the real Rust manager, sparse store, loopback gateway, and
player. It is the end-to-end proof of the 2 MiB store ceiling and progressive
primary-to-mirror rotation. Rust tests independently cover mirror ordering,
focused-store leases, and forced capacity exhaustion and release.

The Android suite uses the real Flutter `video_player` adapter with a
deterministic eight-second fMP4/HLS fixture. It asserts telemetry, playhead
continuity, and absence of visible or semantic error UI. Its two closest
player-visible fault substitutes are deliberately narrower:

- the device-side held-response proxy withholds a media-segment response and
  then releases it; it does not exercise the Rust sparse-store capacity limit;
- the device-side same-URL manifest-retry proxy returns one HTTP 503 and then
  succeeds at the same HLS URL; it does not exercise delivery mirror rotation.

The HLS proxy tests do not turn those substitutes into mirror-rotation or
capacity claims. The progressive Android journey does compile the Rust gateway
with the debug-only `device-integration` adapter: it permits literal loopback
fixture origins while retaining the production outbound policy for every other
address. Release builds do not enable that adapter. The journey therefore
proves the real FFI, manager, sparse store, gateway, and player path, but not the
production SSRF rejection path. Actual mirror rotation and forced capacity
exhaustion remain covered by the browser/Rust paths.

## Stable commands

```sh
VIDEO_USER_E2E_BROWSER=/absolute/path/to/chromium make video-user-e2e-impairments
make video-android-emulator-tests
ANDROID_PHYSICAL_SERIAL=<serial> make video-android-physical-tests
make video-delivery-target-contract-test
```

Browser evidence is retained under `.artifacts/video-user-e2e/run-*`. Android
and physical-device acceptance is not waived when no compatible device is
attached; the blocked run and exact device/toolchain error must be reported.
