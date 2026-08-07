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

## Android — 2026-08-04 (second pass) — feed parity re-measured after the fix

Branch `rust-media-engine` (`a4791ae` plus the branch's uncommitted work — the
`FfiFeedStage` / `RustFeedPageReader` change that makes the adapter wait for a
page to settle, and the `FfiFeedSpec.creators` change that names every decoded
creator). Purpose: decide whether the divergence recorded on 2026-08-04 is gone.

Environment: same AVD `Medium_Phone_API_36.1`, Android 36.1 x86_64, real
outbound network. Native library built with `ANDROID_ABI=x86_64 make
rust-no-clean`, app with `flutter build apk --profile --target-platform
android-x64` (60,0 MB, SHA-256
`709a4bb84cbd098453b13cdbe2873a60728bb40d10057f3a6474aeeb85a0f7d7`, x86_64-only,
`libapp.so` + `libvmservice_snapshot.so` + `librust_lib_ghostr.so` +
`librust_lib_ndk.so` + `libflutter.so` + `libdatastore_shared_counter.so`).
Profile again, not debug: the emulator's 6 GB `/data` had 464 MB free.
`FeedPipelineFlag`'s default was temporarily `FeedPipelineMode.shadow`; the
built `libapp.so` contains `ShadowCompareRemoteVideoSource`, `RustFeedPageReader`
and `FfiFeedStage`, so the tested binary really carries the fix. Signed in with
the public NIP-19 test-vector nsec from `test/nostr/ndk_nostr_session_test.dart`;
no real user key. The flag was reverted afterwards and `git status` on
`lib/app/feed_pipeline_flag.dart` is clean.

How the records were read (reusable):

- `dart:developer.log` never reaches logcat. The app's VM service URI comes out
  of logcat (`The Dart VM service is listening on http://127.0.0.1:<port>/<tok>/`)
  in a profile build; `adb forward tcp:<port> tcp:<port>` then a WebSocket to
  `ws://127.0.0.1:<port>/<tok>/ws` with `streamListen{streamId:"Logging"}`.
- The `LogRecord.message` in a `streamNotify` is an `@Instance` whose
  `valueAsString` the VM truncates at 128 characters, which is why the previous
  pass could only read part of each record. Every record with
  `valueAsStringIsTruncated` was re-read with `getObject(isolateId, objectId,
  offset, count)` in 4096-character chunks until `offset >= length`.
- The deadlock the previous pass hit is avoided by never awaiting inside the
  socket's read loop: one reader task dispatches replies to futures by request
  id and spawns each event handler as its own task, and every request carries an
  8 s timeout. 82 of 82 records came back complete; zero unresolved truncations.

Measured — feed parity in shadow mode:

- **Parity does not hold.** 82 `ghostr.feedparity` divergence records over a
  45-minute session covering three app launches, ~60 feed swipes, 10 deliberate
  search terms (`nostr`, `music` reused from last time, plus `bitcoin`, `art`,
  `dance`, `guitar`, `surf`, `coffee`, `skateboard`, `dog`) and their result
  pages. Previous pass: 7 records.
- Zero `Shadow feed failed` records: the Rust pipeline answered every request
  without throwing, as before.
- Zero `orderMismatches` in all 82 records. Among the ids both pipelines serve
  the ranking still agrees exactly; the disagreement is membership only. This is
  unchanged from the previous pass.
- 70 of 82 records hit the reporter's five-entry cap on at least one list
  (35 on `missing`, 51 on `extra`), so most sizes below are lower bounds of 5,
  not exact counts.
- **Main feed, whenever the ndk truth was healthy: 6 records, every one of them
  `missing=[5 ids]`.** (08:16:16, 08:20:51, 08:26:35, 08:44:36, 08:44:39,
  09:01:39 — the first load of each of the three launches and the reloads in
  between.) That is exactly the previous pass's finding, at the same magnitude.
- Searches: 27 records over the 10 deliberate terms. 12 carry `missing` rows
  (`nostr` 5+5, `guitar` 5+5, `surf` 5+5, `coffee` 5+5 in the first session, and
  `music` 5+5, `guitar` 5+5 again after a restart); 10 are `missing=[]` with the
  Rust side serving 1 to 5 rows ndk did not (`bitcoin` a single id twice, `art`
  four ids twice, `music` and `dance` five ids twice). Repeating the same term
  twice in a row produced byte-identical divergence for 8 of 10 terms, so this is
  a stable pipeline difference, not live-relay noise.
- The one clean comparison seen all session: after the third launch, the first
  main-feed load plus eight swipes produced no record at all. It did not survive
  the next reload (09:01:39 diverged with `missing=[5 ids]`).
- A caution about the remaining 26 `feed` records, all shaped `missing=[]
  extra=[5 ids]` between 08:45:50 and 08:57:22: from 08:45 the ndk truth had
  degraded to serving nothing — the screen showed the "Hunting for videos" empty
  state continuously, and turning "Hide watched videos" off did not refill it,
  while a full app restart did. Those records measure ndk's collapse, not Rust's
  membership, and must not be counted as the Rust pipeline improving. Excluding
  that window, the main feed diverged with `missing=[5 ids]` on 6 of 6 loads.
- Net against the previous pass: the fix changed the *shape* of the divergence
  (a `missing=[]` superset direction that appeared once last time is now common
  on searches) but not its magnitude or its incidence. Every observed request
  except one still diverges, and the defining defect — rows the shipping ndk
  path serves that the Rust shadow does not, at the five-id cap — is still there
  on both the main feed and searches.

Verified — progressive playback did not regress:

- Real frames, not logs: five `screencap`s 0.7 s apart on one post at 08:17 were
  all different (a QR code on a device screen giving way to a menu UI), and the
  same check on a fresh launch at 08:59 gave five different frames of a science
  clip. Video is decoding and advancing.
- Across ~60 swipes at least eight distinct creators played. ExoPlayer
  (`AndroidXMedia3/1.4.1`) bound `c2.goldfish.h264.decoder` to the surface each
  time; zero `PlaybackException`, zero player/source errors, no `Video
  unavailable` panel and no buffering overlay in any screenshot.
- The loopback gateway served the whole session (`127.0.0.1:45017` in the last
  launch).

Not verified:

- Profile feeds. No `profile:` context was ever reported, so the
  `FfiFeedSpec.creators` fix (every creator named instead of only the first) was
  neither confirmed nor refuted here; the Profile tab visits produced only `feed`
  records.
- Hashtag (`tag:`) feeds, older-page parity on search results (the search screen
  does not re-query on scroll), HLS playback, resume-after-kill, cache eviction
  under a full budget, and the arm64 release build.
- Exact divergence sizes. The reporter caps each list at five, and 70 of 82
  records hit that cap, so "5" almost always means "5 or more".

Problems seen:

- The ndk truth pipeline degrades to an empty feed after roughly 30 minutes of
  use and only a process restart recovers it (see the 08:45–08:57 window above).
  This is the shipping path, not the Rust engine.
- `Null check operator used on a null value` at
  `RelayManager.registerRelayRequest (package:ndk/…/relay_manager.dart:341)`
  still reaches the zone handler, together with `could not connect to
  wss://relay.nostr.band` timeouts. Unchanged from the previous pass.
- A chunk-transfer retry storm: 174 identical
  `rust_lib_ghostr::video … Chunk transfer failed: … dns error: failed to lookup
  address information` warnings for one host, `cdn.sovbit.host`, retried every
  ~3 s for ten minutes with no backoff and no give-up.
- Storage. With the default 2 GB video budget the store reached 417 MB and filled
  `/data` to 100% within five minutes of browsing. The store was deleted and the
  budget lowered to 256 MB for the rest of the pass, which held free space
  steady; the "Data usage" level was left at Balanced so relay fan-out — and
  therefore the feed under test — was untouched.
- Installing again needed `sys_storage_threshold_percentage` lowered to 1; it was
  deleted afterwards and reads `null` (the default) again.

The AVD and the two unrelated applications on it were kept. The shadow-mode
profile build was uninstalled at the end so no non-shipping build is left behind,
which also removed its settings and cached video store; `/data` was left with
403 MB free.

## Android — 2026-08-04 (third pass) — parity after outbox routing, warm feeds, retry backoff and the free-space cache

Branch `rust-media-engine` (`a4791ae` plus the branch's uncommitted work).
Purpose: re-measure feed parity now that NIP-65 outbox routing is actually
populated (Rust previously queried only three bootstrap relays while ndk queried
twelve or more), feeds are kept warm instead of cold-opened per pull, retry
backoff replaced the no-give-up retry storm, and the video store respects the
device's real free space. Also to re-check the two defects the second pass
recorded.

Environment: same AVD `Medium_Phone_API_36.1`, Android 36.1 x86_64, real
outbound network. Native library built with `ANDROID_ABI=x86_64 make
rust-no-clean`, app with `flutter build apk --profile --target-platform
android-x64` — 60,012,559 bytes, SHA-256
`f6380c1f8cc49508f0ce6c2cab235dc534a51e6c31ff519977411c1fff8f38f2`, x86_64-only,
`libapp.so` + `libvmservice_snapshot.so` + `librust_lib_ghostr.so` +
`librust_lib_ndk.so` + `libflutter.so` + `libdatastore_shared_counter.so`.
Profile again, not debug: `/data` had 460 MB free. `FeedPipelineFlag`'s default
was temporarily `FeedPipelineMode.shadow`; the built `libapp.so` contains
`ShadowCompareRemoteVideoSource`, `RustFeedPageReader` and `FfiFeedStage`, and
the packaged `librust_lib_ghostr.so` contains `outbox_bootstrap`, so the tested
binary carries the work under test. Signed in with the public NIP-19 test-vector
nsec from `test/nostr/ndk_nostr_session_test.dart`; no real user key. The flag
was reverted afterwards and `git diff lib/app/feed_pipeline_flag.dart` is empty.

Records were read exactly as the second pass documents: VM service URI out of
logcat, `adb forward`, WebSocket `streamListen{streamId:"Logging"}`, and every
`valueAsStringIsTruncated` message re-read with chunked `getObject` while the
socket's read loop never awaits. 60 of 60 records came back complete; zero
unresolved truncations.

Measured — feed parity in shadow mode:

- **Parity still does not hold, and on the defining defect it did not improve.**
  60 `ghostr.feedparity` divergence records over a 53-minute session (10:43 to
  11:34) covering four app launches, about 80 feed swipes, and 42 deliberate
  search submissions over the ten terms the second pass used (`nostr`, `music`,
  `bitcoin`, `art`, `dance`, `guitar`, `surf`, `coffee`, `skateboard`, `dog`),
  each term run three or four times.
- Zero `Shadow feed failed` records: the Rust pipeline answered every request
  without throwing, as in both previous passes.
- Zero `orderMismatches` in all 60 records. Among the ids both pipelines serve
  the ranking still agrees exactly; the disagreement is membership only.
  Unchanged across all three passes.
- **56 of 60 records list rows the ndk truth served and the Rust shadow did not,
  and 55 of those hit the reporter's five-entry cap**, so nearly every one is a
  lower bound of 5, not an exact count. The other four records are the only ones
  with an empty `missing` list. Direction split: `missing` only 18, both lists 38,
  `extra` only 4, agreement 0.
- The five-entry cap was hit on some list in 57 of 60 records (55 on `missing`,
  26 on `extra`).
- **Main feed: 13 records, 12 of them `missing=[5 ids]`.** One (11:03:54) was
  `missing=[] extra=[5 ids]`. Example, the last record of the session:
  `feed: missing=[c253bcfe…, 8d43ffeb…, 1a821a77…, 62839613…, 97294e18…]
  extra=[] order=[]`.
- Deliberate searches: 42 records, 41 of them carrying `missing` rows, every one
  at the cap. Only `dance` at 11:24:05 came back `missing=[] extra=[5 ids]`.
  Five further records came from prefix queries the search screen fires while a
  term is typed (`bitc`, `bitcoi`, `g`, `gu`, `guita`); they are reported apart
  because they are not the terms under test.
- Repeat stability is lower than last pass: of nine terms run twice back to back
  inside one launch, three produced byte-identical divergence (last pass: eight
  of ten). The `missing=[5 ids]` shape, however, repeated every time.
- **The ndk truth never degraded during this pass, so no records were excluded.**
  The second pass had to discard a 12-minute window in which ndk served nothing.
  Here the "Hunting for videos" empty state was never seen: screenshots at 10:43,
  10:44, 10:52, 11:00, 11:04, 11:08, 11:11, 11:19, 11:27 and 11:33 all show ndk
  serving feed posts or search results, and the structural signature of the
  collapse — `missing=[]` with a populated `extra` — appears in only 4 of 60
  records rather than in a sustained run. Excluding those four as a precaution
  changes nothing: the remaining 56 all list rows Rust did not serve.
- Net against the second pass: record count fell from 82 to 60 only because this
  session issued fewer requests. What matters is unchanged or slightly worse. The
  main feed diverged with `missing=[5 ids]` on 12 of 13 loads (last pass: 6 of 6
  healthy loads). Searches went the wrong way: last pass 10 of 27 search records
  were `missing=[]`, here 1 of 42 is. Against a healthy ndk baseline the Rust
  pipeline is still serving fewer rows than the shipping path on almost every
  request, which is the blocker; the extra rows it also serves are not.

Verified — progressive playback did not regress:

- Real frames, not logs: three sets of five `screencap`s 0.7 s apart, at 10:44,
  11:08 and 11:33, gave five distinct frames every time, and the pairs read back
  show genuinely different moments of the same clip (a camera panning across a
  flooded street; a hand moving along a piano keyboard; burned-in subtitles
  advancing from "baked into the money that" to "by design, attacks your").
- Across the session ExoPlayer (`AndroidXMedia3/1.4.1`) bound
  `c2.goldfish.h264.decoder` to the surface on every post; many distinct creators
  played. In the final launch there were zero `ExoPlaybackException`s.

Verified — the second pass's two defects are gone:

- **No retry storm.** `host_stats.json` shows `cdn.sovbit.host` with
  `failure_ratio` 1.0, so the host that produced 174 identical
  `Chunk transfer failed … dns error` warnings every ~3 s for ten minutes last
  pass was hit again — and this time it produced exactly one warning
  (`Probe failed: … dns error: failed to lookup address information`) before the
  source was retired. Giving up is visible as
  `No working source left for <id>; reporting it unplayable`, three times in the
  whole session. Total Rust warnings in the 6-minute fourth launch: 19.
- **The cache no longer runs the disk to 0.** Across 100 samples taken every 30 s,
  `/data` free space never fell below 259 MB of 5.8 GB, and the store peaked at
  143 MB. The guard is visible in the log
  (`Video store gave back 95743110 of 4956160 bytes to protect free space`) and
  in `partial_range_store/capacity.rs`, where `DEFAULT_RESERVE_BYTES` is 256 MB
  and the effective cap is `min(budget, used + free − reserve)`. Last pass the
  same browsing filled `/data` to 100% within five minutes with a 417 MB store,
  and the budget had to be hand-lowered to 256 MB; this pass needed no
  intervention and the budget was left alone.

Not verified:

- Exact divergence sizes. The reporter caps each list at five and 57 of 60
  records hit that cap, so "5" almost always means "5 or more". Nothing in the
  record says how many rows either pipeline returned in total.
- Profile feeds again: no `profile:` context was ever reported even after
  visiting the Profile tab, so the `FfiFeedSpec.creators` fix is still neither
  confirmed nor refuted on device.
- Hashtag (`tag:`) feeds, older-page parity on search results, HLS playback,
  resume-after-kill of a partial download, and the arm64 release build.

Problems seen:

- **An out-of-memory crash.** At 10:45:51, about three minutes and ten swipes
  into the first launch, the Java heap hit its 192 MB growth limit and the
  process died: repeated `OutOfMemoryError` from
  `androidx.media3.exoplayer.upstream.DefaultAllocator.allocate` via
  `SampleDataQueue.preAppend` on several concurrent
  `ExoPlayer:Loader:ProgressiveMediaPeriod` threads, then
  `FlutterJNI` `Check failed: fml::jni::CheckException(env)` and
  `Fatal signal 6 (SIGABRT)`. It happened once in the session and not again over
  the following 45 minutes. This was a shadow build, which runs both discovery
  pipelines in one process, so the crash cannot be attributed to the Rust engine
  alone — but the allocation that failed was ExoPlayer's sample buffer, not feed
  data.
- The store now refuses writes in bursts once free space reaches the 256 MB
  reserve — 16 `Chunk transfer failed: video store is out of space: N bytes short`
  lines inside one second at 11:29, and similar bursts at 11:11 and 11:19. Around
  the 11:19 burst one post showed the `Video unavailable` panel, and seven
  `Video player initialization failed` records were logged over the session. The
  second pass reported no such panel. Refusing to write is the correct half of
  the fix; the user-visible consequence of refusing is not obviously handled.
- The progressive store does not survive a process restart. After every launch
  the store measured 8 KB and free space returned to about 400 MB, while
  `host_stats.json` in the same directory persisted. Nothing downloaded in one
  session is reused in the next.
- The main feed stopped advancing in the second launch: eighteen consecutive
  swipes over four minutes left the same post on screen, with no older-page pull
  and no new parity record. A relaunch recovered it.
- The ndk path's own faults are unchanged: 44 `could not connect to wss://…`
  timeouts (`relay.nostr.band` 23, `relay.ditto.pub` 13, `relay.damus.io` 7,
  `relay.snort.social` 1) and 16 unhandled `Null check operator used on a null
  value` exceptions from `RelayManager.registerRelayRequest` in the last launch
  alone.
- Installing again needed `sys_storage_threshold_percentage` lowered to 1; it was
  deleted afterwards and reads `null` (the default) again.

The AVD and the two unrelated applications on it were kept. The shadow-mode
profile build was uninstalled at the end so no non-shipping build is left behind,
which also removed its settings and cached video store; `/data` was left with
403 MB free.

## Android — 2026-08-04 (fourth pass) — parity after the session event pool, the store reload and the eviction-first space guard

Branch `rust-media-engine` (`c28ed04` plus the branch's uncommitted work).
Purpose: re-measure feed parity now that the Rust Nostr client keeps its own
event database, so a query answers stored UNION network the way ndk's
`MemCacheManager` does — the root cause the previous three passes kept
measuring. Also to re-check the three defects the third pass recorded: the
progressive store not surviving a restart, the burst of store write-refusals
with its `Video unavailable` panel, and the one-off ExoPlayer
`OutOfMemoryError`.

Environment: same AVD `Medium_Phone_API_36.1`, Android 36.1 x86_64, real
outbound network. Native library built with `ANDROID_ABI=x86_64 make
rust-no-clean`, app with `flutter build apk --profile --target-platform
android-x64` — 60,129,167 bytes, SHA-256
`43e41225a177de3372ac2be5c593c96d929705e71412509690b0d3491ff228fe`,
x86_64-only, `libapp.so` + `libvmservice_snapshot.so` + `librust_lib_ghostr.so`
+ `librust_lib_ndk.so` + `libflutter.so` + `libdatastore_shared_counter.so`.
Profile again, not debug: `/data` had 403 MB free. `FeedPipelineFlag`'s default
was temporarily `FeedPipelineMode.shadow`. The tested binary really carries the
work: the packaged `librust_lib_ghostr.so` (11,477,008 bytes) contains
`rust_lib_ghostr::discovery::event_cache`, `src/discovery/event_cache.rs`,
`The session event pool could not be read/cleared/store an event`,
`Video store could not reload`, `Video store could not clear unusable` and
`Video store has no room for … pausing the post instead of its source`, and
`libapp.so` contains `ShadowCompareRemoteVideoSource`, `RustFeedPageReader` and
`FfiFeedStage`. Signed in with the public NIP-19 test-vector nsec from
`test/nostr/ndk_nostr_session_test.dart`; no real user key. The flag was
reverted afterwards and `git diff lib/app/feed_pipeline_flag.dart` is empty.

Session shape, chosen for comparability with the third pass and to give a
session-accumulating cache a fair chance to warm: 12:57 to 14:31, four app
launches, ~270 feed swipes, 20 main-feed loads (one cold load per launch plus
16 forced Profile→Home rebuilds), and 80 deliberate search submissions over the
same ten terms (`nostr`, `music`, `bitcoin`, `art`, `dance`, `guitar`, `surf`,
`coffee`, `skateboard`, `dog`), six to nine repetitions of each. Every launch
browsed the feed for four to seven minutes *before* its search sweep, and the
sweeps were repeated as the session aged.

Records were read exactly as the second and third passes document: VM service
URI out of logcat, `adb forward`, WebSocket `streamListen{streamId:"Logging"}`,
every `valueAsStringIsTruncated` message re-read with chunked `getObject`, and
the socket's read loop never awaiting. 130 of 130 records came back complete;
zero unresolved truncations. Swiping an already-buffered feed issues no pull, so
main-feed data points come from launches and forced reloads, not from swipes.

Measured — feed parity in shadow mode:

- **Parity still does not hold, and on the defining defect it did not improve.**
  130 `ghostr.feedparity` divergence records; **28 excluded** because the ndk
  truth was unhealthy (below), leaving **102 against a healthy ndk truth**.
- Zero `Shadow feed failed` records: the Rust pipeline answered every request
  without throwing, as in all three previous passes.
- Zero `orderMismatches` in all 130 records. Among the ids both pipelines serve
  the ranking still agrees exactly; the disagreement is membership only.
  Unchanged across all four passes.
- **97 of the 102 healthy records list rows the ndk truth served and the Rust
  shadow did not, and 96 of those hit the reporter's five-entry cap.** Third
  pass: 56 of 60, 55 at the cap. Direction split: `missing` only 8, both lists
  89, `extra` only 5, agreement 0. The cap was hit on some list in 101 of 102.
- **Main feed: 28 healthy records, 27 of them carrying `missing` rows, 26 at the
  cap.** Third pass: 12 of 13. Every one of the 20 deliberate loads diverged.
- **Deliberate searches: 73 healthy records from 70 healthy submissions, 70 of
  them carrying `missing` rows, every one at the cap.** Third pass: 41 of 42.
  Two further records came from prefix queries the search screen fires while a
  term is typed (`skateboar`, `d`); they are reported apart.
- Repeat stability: each term was run six to nine times across the session and
  diverged every single time; per term there were only two to six distinct
  `missing` sets, and reruns inside one launch were frequently byte-identical.
- **The trend with session age is flat.** Mean `|missing|` per ten-minute
  bucket, healthy records only: 5.00, 5.00, 5.00, 5.00, 3.57, 4.75, 4.38, 5.00,
  5.00, 4.69. No decay. Per launch, `missing` rows appear in 7 of 7, 7 of 8, 5
  of 5 and 8 of 8 main-feed records — a two-hour-old pool is no better than a
  two-minute-old one.
- The pool is nevertheless demonstrably live, and it moved the *other*
  direction. Mean `|extra|` went from 0.00 on the session's first cold load to
  4.2–5.0 within ten minutes and stayed there. The clearest single proof: the
  13:08:06 main-feed record's `extra` list is exactly the five ids the 12:58:35
  record had listed as `missing` — rows Rust could not serve on the cold query,
  served ten minutes later out of the pool, by which time the ndk truth had
  rotated them out. The fix does what it says; it just does not reach the rows
  ndk has and Rust never fetched.
- Net against the third pass: the record count rose from 60 to 130 because this
  session issued more requests. The rates that matter are unchanged or slightly
  worse — 95% of healthy records short (third pass 93%), main feed 27 of 28
  (12 of 13), searches 70 of 73 (41 of 42). Against a healthy ndk baseline the
  Rust pipeline is still serving fewer rows than the shipping path on almost
  every request. The extra rows it also serves are not the blocker.

Excluded records — the ndk truth collapsed again:

- From 14:08:04 to 14:16:01, spanning the end of the third launch and the first
  minute of the fourth, every record is `missing=[] extra=[5 ids]`: 28 records
  (17 `feed`, 10 search, 1 prefix). Screenshots at 14:06:40, 14:08:45 and
  14:09:45 show the "Hunting for videos" empty state, and the search screen at
  14:14:29 shows "No matches found" for `dog`, a term that had returned eight
  videos at 13:17. A process restart recovered it, as in the second pass. Those
  records measure ndk's collapse, not Rust's membership.
- The collapse arrived after roughly eight minutes of the third launch, not the
  ~30 minutes the second pass saw. Everything outside that window is counted.

Verified — progressive playback did not regress:

- Real frames, not logs: three sets of five `screencap`s 0.7 s apart, at 13:07,
  13:53 and 14:29, gave five distinct frames every time, and the pairs read back
  show genuinely different moments of the same clip (a dog running from the left
  edge of a crosswalk into the middle of it; a camera panning across two lions
  so the framing and their heads move; an animated masked figure turning its
  head and shifting its arms).
- ExoPlayer (`AndroidXMedia3/1.4.1`) bound `c2.goldfish.h264.decoder` to the
  surface throughout — 243 `ExoPlayerImpl` inits over the session — and the
  loopback gateway served every launch (`127.0.0.1:44317`, `:39293`, `:42775`,
  `:38771`).

Verified — the free-space guard now evicts instead of refusing:

- **Zero store refusals in 94 minutes.** No `video store is out of space` and no
  `Video store has no room for … pausing the post instead of its source` line
  was logged at all. Third pass: sixteen refusals inside one second at 11:29,
  with further bursts at 11:11 and 11:19.
- 181 disk samples taken every 30 s. Free space bottomed at exactly 256 MB — the
  `DEFAULT_RESERVE_BYTES` line — at 14:22:17 with the store at 149 MB, and
  recovered; earlier the store cycled 147 → 131 → 105 MB while free space
  climbed back from 265 to 307 MB. Eviction on the write path is silent by
  design (`partial_range_store/admission.rs`, `make_room`), and the timer path's
  `Video store gave back …` never had to fire.

Verified — the third pass's `OutOfMemoryError` did not recur:

- Zero `OutOfMemoryError`, zero `Fatal signal`, zero `FATAL EXCEPTION` across
  four launches, 243 player inits and 94 minutes.

Not verified — and one fix that does not reach the device:

- **The progressive store still does not survive a restart, and the Rust half of
  the fix never runs.** Before the second launch the store held 66 files /
  105,152 KB; ten seconds after it, `files/native_video_inventory` was empty —
  no `progressive/`, no `host_stats.json`. Before the third launch: 46 files /
  144,932 KB → 8 KB. The sampler reads `store_kb=8` at 12:58:41, 13:30:24 and
  14:01:09, the same signature the third pass recorded. The cause is on the Dart
  side, not in Rust: `lib/platform/media/native_video_cache_directory.dart`'s
  `initialize()` still runs `if (await directory.exists()) await
  directory.delete(recursive: true);`, and
  `lib/app/production_video_delivery_infrastructure.dart` calls it immediately
  before `gateway.start(...)`, so `PartialRangeStore::load_existing`
  (`rust/crates/media-store/src/partial_range_store/reload.rs`, called from
  `rust/crates/gateway/src/gateway_delivery.rs`) always adopts an empty
  directory.
  `prepare_native_cache_directory` was correctly narrowed to sweep only stale
  `*.mp4` / `*.partial`, but the directory is gone before it looks.
- Exact divergence sizes, again: the reporter caps each list at five and 101 of
  102 healthy records hit that cap, so "5" almost always means "5 or more".
- Profile feeds, for the third pass running: no `profile:` context was ever
  reported.
- Hashtag (`tag:`) feeds, older-page parity on search results, HLS playback,
  resume-after-kill, and the arm64 release build.

Problems seen:

- **`Video unavailable` panels recurred, and more often than last pass.** Caught
  in screenshots at 13:22, 13:34, 13:37 and 14:04, with 57 `Video player
  initialization failed` records over the session (third pass: seven). They are
  *not* store refusals this time — there were none. logcat shows 57
  `ExoPlaybackException: Source error`, of which 39 are
  `UnrecognizedInputFormatException` ("none of the available extractors could
  read the stream") and 15 are `InvalidResponseCodeException` (404 or 503 from
  the loopback gateway). At least one panel sat on a post whose content is
  `https://image.nostr.build/b1d733a0…` — a still image admitted into a video
  feed, which no extractor can read.
- 10 `cannot write into a finalized video` warnings, new this pass.
- 15 `No working source left for <id>; reporting it unplayable`, 78
  `Chunk transfer failed` and 11 `Probe failed` — retry backoff is still holding
  (no storm), and `Video store could not reload` never appeared.
- The ndk path's own faults are unchanged and, on this run, louder: 287
  `could not connect to wss://…` timeouts (`relay.nostr.band` 181,
  `relay.ditto.pub` 81, `relay.damus.io` 25) and 40 unhandled `Null check
  operator used on a null value` exceptions from
  `RelayManager.registerRelayRequest`.
- Installing again needed `sys_storage_threshold_percentage` lowered to 1; it
  was deleted afterwards and reads `null` (the default) again.

The AVD and the two unrelated applications on it were kept. The shadow-mode
profile build was uninstalled at the end so no non-shipping build is left
behind, which also removed its settings and cached video store; `/data` was left
with 415 MB free.

## Android — 2026-08-05 — Rust Nostr engine cutover

Purpose: verify the completed production cutover with Rust as the sole relay
communication engine, including account activation, live feed/search reads, and
account reset on sign-out.

Environment: Android 17 / API 37 x86_64 emulator (`emulator-5580`, 1080x2400,
14 GB free under `/data`). The native library was built with
`ANDROID_ABI=x86_64 make rust-no-clean`; `make android-debug-apk-check` built and
validated an x86_64-only debug APK. The APK is 104,411,495 bytes with SHA-256
`86b30fac938c6c27f706fa025d658774069944eb3f47e595d02a98b19029c8a1`.
Its four packaged native libraries are `libdatastore_shared_counter.so`,
`libflutter.so`, `librust_lib_ghostr.so`, and `librust_lib_ndk.so`.

The app was freshly installed after confirming `app.ghostr` was absent. Only
Ghostr's newly installed app data was reset while preparing the run; unrelated
applications and emulator data were not changed. Sign-in used the repository's
public NIP-19 test-vector nsec, never a real user key, and no event was
published.

Verified:

- A cold launch rendered the password-protected Nostr-key import screen.
- The public test identity activated successfully and opened the authenticated
  five-tab application shell.
- The main feed left `Loading video feed` and populated from live relays with a
  playable Nostr video row.
- The search tab populated live trending tags. Searching for `dog` returned
  both creator matches and video matches, without an error or unsafe empty
  state.
- The profile read resolved the test identity as `Nostr 0ELF`, including its
  public npub and 60 followed accounts.
- Sign-out returned immediately to the key-import screen. Force-stopping and
  cold-launching the app again stayed signed out, confirming the persisted
  session and Rust account scope were reset.
- A process-scoped logcat scan found no Rust panic, fatal exception, unhandled
  Flutter exception, segmentation fault, or crash during sign-in, feed,
  search, profile, sign-out, and cold restart.

One Android platform advisory repeated in logcat:
`AIBinder_linkToDeath ... no onUnlink callback ... will become an abort`.
There was no actual abort or user-visible failure, and the app completed the
entire smoke flow.

### Post-review rebuild and cold-start re-smoke

After the final review fixes, the x86_64 native library and debug APK were
rebuilt with `ANDROID_ABI=x86_64 make rust-no-clean` and
`make android-debug-apk-check`. The ABI-validated APK is 104,411,495 bytes with
SHA-256
`e129d17d9e28ee6105ce251de226982ef593b6701be993272e6ff53fccb136e6`.

The APK installed successfully on `emulator-5580`. A forced cold launch
completed in 5,146 ms, the process stayed alive, and the password-protected
Nostr key-import screen rendered while the previously verified signed-out
state remained cleared. A launch-scoped AndroidRuntime, Flutter, Rust, libc,
and native-debug error scan was empty.
