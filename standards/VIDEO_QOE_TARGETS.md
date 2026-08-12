# Video QoE Targets

These are release gates for the adaptive video-delivery path. They are SLOs,
not a claim that playback can be literally uninterruptible on every network or
device.

## Automated acceptance budgets

| Metric | Target | Measurement |
| --- | ---: | --- |
| Ordered warm-prefetch readiness | `<= 4,000 ms` | On a fresh cache with focus held on the first video, the active video and next three each hold at least `48 KiB`. |
| Ordered far-ahead origin use | `0 B`, `0 starts` | During that complete held-focus scenario, videos beyond the first four have no body request start and receive no origin body bytes. |
| Cold startup latency | `<= 2,000 ms` | User focus intent to the first `playing` observation for that video. |
| Focus-switch latency | `<= 1,500 ms` | Each non-superseded focus intent to `playing` for its destination. |
| Rebuffer ratio | `<= 1%` | Time in `buffering` or network-stalled phases after first play, divided by observed playback time. |
| Cancellation waste | `<= 192 KiB` | Origin body bytes sent after focus left a request that was then canceled. |
| Ahead prefetch | `48 KiB` to `3 MiB` | Maximum stored sibling bytes gained while the current video is active and incomplete. |
| Far-ahead before frontier | `0 B` | Exact origin body bytes sent for videos beyond active plus next three before each protected video receives its first `48 KiB`. |
| Far-ahead starts before frontier | `0` | Origin body requests started for videos beyond active plus next three before that protected frontier. |
| Duplicate completed origin bytes | `0 B` | Successful completed body-range overlap previously fetched from the same exact source. |
| Protected transition latency | `<= 500 ms` | Tagged current-to-next transitions from click intent to the first `playing` observation. |
| Held-response recovery | `<= 2,000 ms` | Device fixture response release to resumed `playing`. |
| Manifest-retry startup | `<= 4,000 ms` | Focus through one same-URL manifest HTTP 503 to first `playing`. |

The browser contract is defined in `tool/video_user_e2e/qoe_targets.mjs`.
Device constants live in
`integration_test/support/device_qoe_targets.dart`. A change to either budget
requires a failing contract test and an explanation backed by retained traces.

The browser matrix admits eight ordered videos, but splits prefetch readiness
from moving-focus QoE. The `ordered_prefetch` row starts with a fresh private
cache, explicitly focuses the first video, and holds that focus without a
player click transition. From the pre-focus stored-byte baseline it requires
videos zero through three to reach `48 KiB` within four seconds. Across the
entire row, including a fixed-focus `500 ms` observation after readiness,
exact shared-origin start and chunk evidence must show zero body request starts
and zero body bytes for video four or later.

Every impairment row runs in its own process with another fresh private cache;
it does not inherit the ordered-prefetch row's store and does not wait for a
fixed initial warm-up before moving focus. Before the initial focus selection,
and immediately before every trusted click, it captures a stored-byte baseline
and an exact shared-origin ordinal boundary. Each focus epoch protects its
current video plus the next three available videos, including shorter windows
at the feed tail, until the next click. Existing protected bytes count as
readiness. Global origin start/chunk ordinals are replayed within every epoch to
measure bandwidth order, while store snapshots separately prove retained
readiness. Live origin evidence is authoritative and missing, duplicate, or
malformed focus-boundary ordinals fail closed; UI fallback is reserved for
synthetic traces with no `origin_requests` property.

Its progressive fixture is a valid six-second H.264 MP4 whose timed samples
span `285,652` bytes and several `64 KiB` transfers. The origin and registration
advertise that exact size and duration; no virtual zero tail is counted as
media. Eight fixtures also exceed the `2 MiB` pressure budget.

## Deterministic impairment matrix

The browser acceptance suite covers:

- held-focus ordered prefetch on a fresh cache, with exact zero origin use past video three;
- one shared link stepping from 2.5 Mbps to 700 Kbps, then recovering to 2.5 Mbps;
- deterministic failure of the first two `v2` body attempts after 128 KiB;
- 450 ms RTT at 2.5 Mbps;
- four focus changes at 200 ms intervals;
- a 2 MiB storage ceiling followed by an explicit capacity release;
- an HTTP 503 from the primary progressive source with a healthy mirror.
- protected-prefix transitions at 2.5 Mbps, 100 ms RTT, and one host connection.

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
by a trace-verified recovery profile. High RTT must be sampled during active,
incomplete delivery at exactly 2.5 Mbps, 450 ms, and three host connections.
Protected transitions likewise require a sampled active transfer at exactly
2.5 Mbps, 100 ms, and one host connection; bootstrap control receipts alone do
not satisfy either activation guard.

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

No mobile loopback bypass or production test hook is used to turn these proxies
into false end-to-end claims. Actual mirror rotation and store capacity remain
covered by the browser/Rust paths, while device execution proves how the real
player surfaces and recovers from their representative playback effects.

## Stable commands

```sh
make video-user-e2e-impairments
make video-android-emulator-tests
ANDROID_PHYSICAL_SERIAL=<serial> make video-android-physical-tests
make video-delivery-target-contract-test
```

Browser evidence is retained under `.artifacts/video-user-e2e/run-*`. Android
and physical-device acceptance is not waived when no compatible device is
attached; the blocked run and exact device/toolchain error must be reported.
