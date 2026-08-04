# Manual verification

## Android — 2026-08-02

Environment: disposable Android 36.1 x86_64 emulator, using the debug APK.

Verified:

- The sign-in screen rendered and treated the nsec field as a password.
- A deterministic test nsec opened the authenticated application shell.
- Reinstalling the APK restored the authenticated session from secure storage.
- Feed, search, compose, activity, and profile navigation remained available.
- Compose exposed both gallery and camera actions while Publish stayed disabled
  without a selected draft.
- Denying camera access returned the app-safe permission message and kept
  Publish disabled.
- Granting camera access opened Android's camera in video mode; the capture was
  cancelled without recording media.
- Choosing the gallery opened Android's system photo picker; the selection was
  cancelled without granting the app access to user media.
- The current-user profile showed the derived Nostr identity, sign-out action,
  and settings entry.
- Settings showed editable relay and Blossom endpoints and the selected 2 GB
  video inventory budget.
- The packaged Rust gateway started without the prior unavailable-gateway
  state. The feed showed its normal loading state, and logs contained no Dart,
  Flutter, or Rust panic/exception.

The disposable emulator had no external network access, so live relay feed
population, remote video playback, Blossom upload, and publication were not
manually exercised. Automated adapter, widget, cache, fallback, upload,
publication, and native gateway tests cover those boundaries.

After the typed Rust startup bridge was regenerated, packaging was reverified:

- The debug APK is x86_64-only. Its four native libraries are
  `libdatastore_shared_counter.so`, `libflutter.so`,
  `librust_lib_ghostr.so`, and `librust_lib_ndk.so` under `lib/x86_64/`.
  The APK is 121,126,661 bytes, its packaged `librust_lib_ghostr.so` is
  20,821,392 bytes, and its SHA-256 is
  `d73bc6fb1916973ae7d0b6ee0a3b71ea53f98bf2b4cf5095a16801a7cf6d9bb6`.
- The release APK is arm64-v8a-only. Its five native libraries are `libapp.so`,
  `libdatastore_shared_counter.so`, `libflutter.so`,
  `librust_lib_ghostr.so`, and `librust_lib_ndk.so` under `lib/arm64-v8a/`.
  The APK is 30,510,286 bytes, its packaged `librust_lib_ghostr.so` is
  8,567,064 bytes, and its SHA-256 is
  `6db952cdc5f38602666b5c26e8df60ae8f73e14def6a8261a34ee82d9b97932f`.

The disposable AVD was deleted after verification. No unrelated applications
or user data were removed.

## Android — 2026-08-04 — Rust media engine, progressive playback

Branch `rust-media-engine` (`526079b` plus the branch's uncommitted work).
First device pass for plan §4 step 6 and the first risk in §8: whether
ExoPlayer streams progressive MP4 from the Rust loopback gateway out of a
partially downloaded sparse file.

Environment: Android 36.1 x86_64 emulator (AVD `Medium_Phone_API_36.1`,
1080x2400), with real outbound network — unlike the 2026-08-02 pass. Native
library built with `ANDROID_ABI=x86_64 make rust-no-clean`, app with
`make android-debug-apk` (`--target-platform android-x64`). Signed in with the
public NIP-19 test-vector nsec already used by
`test/nostr/ndk_nostr_session_test.dart`; no real user key was used.

Verified — progressive playback through the loopback gateway:

- The feed populated from live relays and video rendered. Five consecutive
  `screencap` frames all differed, so playback advanced rather than freezing on
  a first frame. At least six distinct videos from different creators played
  across roughly twelve swipes and eight minutes.
- ExoPlayer (`AndroidXMedia3/1.4.1`) drove real decoding: async `MediaCodec`
  adapter created for `video/avc`, `c2.goldfish.h264.decoder` bound to the
  surface. On a cold cache, first player init to codec-on-surface was about
  1.8 s.
- The gateway really is the source. It listened on `127.0.0.1:43595`, and while
  video played `/proc/net/tcp` showed three to four simultaneously ESTABLISHED
  connections from the app's uid to that port.
- Header semantics, checked directly over `adb forward`:
  - full GET returns `200`, `content-type: video/mp4`,
    `content-length: 9390756`, `accept-ranges: bytes`; the body delivered
    exactly 9,390,756 bytes and began with `ftyp isom … avc1 mp41`;
  - `Range: bytes=0-1023` returns `206` with
    `content-range: bytes 0-1023/9390756` and `content-length: 1024`;
  - `Range: bytes=1000000-` returns `206` with
    `content-range: bytes 1000000-9390755/9390756`;
  - `Range: bytes=99999999-` returns `416` with `content-range: bytes */9390756`;
  - an unregistered post id returns `404`.
- Serving out of a genuinely sparse file, which was the actual risk. With the
  manifest at
  `{"total_len":1754617,"ranges":[[0,311296],[1048576,1228800],[1492473,1671168]]}`,
  a request for `Range: bytes=311296-811296` — starting exactly inside a hole —
  returned `206`, `content-range: bytes 311296-811296/1754617`, and delivered
  all 500,001 bytes in 526 ms as the missing bytes arrived. The SHA-256 of the
  served bytes equals the SHA-256 of the same span read off the completed file
  afterwards (`e45bc8b17163ace7767cbbefea0aad9df9af1a3f827e5425e02b37d3abe7b0a4`),
  so nothing was zero-filled, duplicated, or misaligned across the hole.
- The scheduler's shape is visible on disk: head chunks capped at 1,250,000
  bytes, a 1 MB chunk grid, and a ~256 KB tail chunk fetched before the middle
  (for example `[[0,2738],[1048576,1049945],[19839668,19841037]]` on a
  20,101,812 byte file), which is the moov-at-end tail probe. Chunk work paused
  while a post left the focus window and resumed later, and every partial file
  observed eventually filled to `[[0,total]]`.
- Prefetch pays off on scroll. From a logcat marker at the swipe, a new
  `ExoPlayerImpl` init landed at +527 ms, its video codec adapter at +820 ms and
  the surface at +1168 ms; a screenshot taken mid-transition shows the outgoing
  and incoming videos both rendering live frames, so the incoming video was
  already started from prefetched head bytes.
- No playback failures: zero ExoPlayer player/source errors, no
  `Video unavailable` panel, no buffering overlay caught in any screenshot,
  clean `Init`/`Release` pairing, and no Rust `warn!` lines (the Rust side logs
  only on failure, so silence means no chunk, probe, store, or finalize error).
- `host_stats.json` persisted a per-host EWMA model: `v.nostr.build`
  653 kB/s / TTFB 1053 ms, `blossom.primal.net` 668 kB/s / 1395 ms,
  `video.nostr.build` 269 kB/s / 1789 ms, one `blossom.band` host 567 kB/s;
  `failure_ratio` 0.0 on all four.

Verified — feed parity in shadow mode (plan §5 step 6):

- With `FeedPipelineFlag`'s default temporarily set to `FeedPipelineMode.shadow`
  and the app rebuilt, seven `ghostr.feedparity` divergence records were
  captured off the Dart VM service `Logging` stream (`dart:developer.log` does
  not reach logcat). Zero `Shadow feed failed` records: the Rust pipeline
  answered every request without throwing.
- The two pipelines do **not** agree yet. Divergence appeared on both the main
  feed and search feeds, almost always as rows the ndk truth served and the Rust
  shadow did not, for example
  `search:to …: missing=[05ec0b5e…, 4eab0f91…, f7799706…, 8f7adf3c…, 1124ce41…]
  extra=[] order=[]` (the reporter caps each list at five, so these are lower
  bounds). One sample went the other way,
  `missing=[] extra=[a9253a1c…] order=[]`. Every fully readable sample had
  `order=[]`, so among the ids both pipelines served the ranking agreed; the
  divergence is membership, not ordering.
- The shadow run used a profile APK because the emulator's 6 GB `/data` could
  not hold a second 147 MB debug APK. Playback stayed healthy in that build too.

Not verified:

- Time-to-first-frame was measured from logcat milestones, not from a frame
  capture, so the numbers above bound it rather than pin it.
- Stall and rebuffer behaviour under a slow or lossy network was not exercised;
  the emulator's link was fast enough that head chunks and whole files usually
  landed quickly.
- HLS playback, resume-after-kill of a partial download, cache eviction under a
  full budget, and the arm64 release build were not exercised.
- Whether the main feed reaches parity after the divergences above were fixed —
  only the presence of divergence was established, not its root cause.

Problems seen:

- The ndk discovery path logged repeated
  `could not connect to wss://relay.damus.io` / `relay.nostr.band` /
  `relay.ditto.pub` timeouts, and an unhandled Dart exception
  `Null check operator used on a null value` at
  `RelayManager.registerRelayRequest (package:ndk/…/relay_manager.dart:341)`
  reached the zone handler more than once. The feed still populated from the
  remaining relays. This is in the shipping ndk path, not the Rust engine.
- A byte-complete partial file is never promoted to `.video` when the note
  advertises no imeta `x`, because `try_finalize` returns early without a
  digest. Five files sat byte-complete as `.part`. Serving from `.part` works,
  so this is not a playback defect, but the finalize/verify branch of the store
  is rarely reached in the wild.
- `MediaCodecRenderer` warned `Format exceeds selected codec's capabilities` for
  a 2172x2052 at 59.63 fps stream on the emulator's `c2.goldfish.h264.decoder`.
  It played anyway; this is an emulator codec limit, not an app defect.
- Installing the debug APK needed the emulator's low-storage threshold lowered
  (`sys_storage_threshold_percentage`), since the APK is 147 MB and the cached
  video store grew past 120 MB on a 6 GB `/data`. Both settings were restored
  afterwards.

The existing AVD and the two unrelated applications already installed on it were
kept; only Ghostr's own package and its cached video store were removed and
reinstalled during this pass.
