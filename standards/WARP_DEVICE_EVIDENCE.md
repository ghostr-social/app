# WARP device evidence

Every entry below was produced by `tool/run_warp_evidence.sh` (raw `stdout.log` and
`logcat.log` are git-ignored; `command.txt`, `commit.txt`, `device.txt`, `exit.txt`,
`markers.log` and `summary.txt` are committed under `evidence/warp/`). Regenerate a section with
`tool/summarize_warp_evidence.sh evidence/warp/<run>`.

Rules for adding a run:

- Run the mandated commands through the wrapper (`make video-android-physical-evidence` wraps
  `video-android-physical-tests` for the physical device) so the commit, device and exit status
  are captured with the output.
- Report p50/p95/p99 only when the sample size and method are stated next to the number.
- A failing run stays in this file with its diagnosis; it is never replaced by an emulator result.

## Device

- serial `22e0d933`, USB, `ro.product.model=M2012K11AG`, Android 13, `ro.kernel.qemu` empty
  (physical hardware; verified with `adb devices -l` and `getprop` before every run).
- Build environment note: with `build/` symlinked onto the external SSD every integration test
  file cost ~26 min (Gradle re-ran the Rust cross-compile); with `build/` as a plain directory on
  the internal disk a rebuild takes ~15 s and the 26-file matrix runs in 18 min.

## Status summary (2026-09-02)

| run | head | result |
|---|---|---|
| `make video-delivery-target-contract-test` | `1c03551b` | pass |
| 26-file matrix (first attempt, `build/` on the external SSD) | `1c03551b` | stopped after 12 files (4.4 h): 11 pass, `adaptive_warm_back` fail |
| targeted validation of the ETA and origin-exhaustion slices (4 files) | `6d015337` | `adaptive_warm_back` pass, `progressive_delivery` pass, `player_verified_rescue` fail on a stale expected reason (fixed in `301db545`), `origin_timeout_fallback` fail at stage 1 |
| bisect of `origin_timeout_fallback` | `4d983a42` (pre-slices) | stage 1 pass, stage 3 fail (`Expected 0, Actual 16384`): pre-existing red test |
| `origin_timeout_fallback` after ETA bucketing | `e1c61852` | stage 1 pass again; only the pre-existing stage-3 assertion fails |
| 26-file matrix (second attempt, `build/` on the internal disk, 18 min) | `e1c61852` | 19 pass, 7 fail (table below) |
| `adaptive_warm_back` alone, twice each on `e1c61852` and `6d015337` | both | pass (intermittent in longer sequences) |
| `make video-android-offline-restart` (seed, force-stop, restore) | `e1c61852` | pass (both stages) |
| `make video-android-lifecycle` (HOME, relaunch, decode) | `e1c61852` | fail: the app went to background and the system reports `topResumedActivity=app.ghostr/MainActivity` after relaunch, but the test never printed `WARP_ANDROID_LIFECYCLE_RESUMED` within 30 s; inherited from `7c08ccc5`, never green |

### The seven matrix failures on `e1c61852`

| file | first assertion | classification |
|---|---|---|
| `warp_feed_adaptive_warm_back` | no `VideoPlayer` mounted under a retained page key | intermittent: passes alone on this head and on `6d015337`, fails after other files in the same installation; no transport rescue in its focus trace |
| `warp_feed_player_verified_rescue` | "WARP candidate evidence timed out; gateTimedOut=false" | intermittent: passes alone (`targeted-rerun`, `isolation-warm-back-and-rescue`) |
| `warp_feed_origin_timeout_fallback` | stage 3, `Expected: <0> Actual: <16384>` | **re-diagnosed 2026-09-02**: not an engine gap. Logcat shows the primary's Transient chunk failure at +15 s (idle deadline), i.e. the client dropped the held socket before release; the fixture counted one write the kernel accepted into the half-closed socket. Fixed by peer-close accounting in the origin fixture and a stage-3 claim that the drop precedes release (proven: peer closed at 16.6 s, fallback started at 16.7 s). Stage 1 (`waitForVerifiedFallback`) remains intermittent (3 of 6 runs): after the full fallback body lands, the reserve player for that post is re-created (~17.6 s) and does not render within 25 s. Next gap to investigate. |
| `warp_feed_long_session_boundedness` | `warp_long_session_scenario_assertions.dart:13` expected false, got true | inherited from the previous agent's uncommitted slice (`7c08ccc5`); never green |
| `warp_feed_cache_pressure` | journey timed out after 30 s (handoffs=3, requests=12, active={}) | inherited, never green |
| `warp_feed_unsupported_hls_rescue` | `warp_unsupported_hls_rescue_scenario_assertions.dart:72` expected false, got true | inherited, never green |
| `warp_feed_stale_validator_rotation` | rotation timed out after 30 s (held=true, requests=4) | inherited, never green |

The 19 passing files on `e1c61852`: bandwidth_drop, packet_loss, high_rtt, rapid_swipes,
held_response, manifest_retry, progressive_delivery, progressive_loop_reopen, feed_playback,
visible_motion, rapid_swipe_instrumentation, startup_singleflight, mixed_hls_readiness,
bandwidth_recovery, ignored_range, malformed_range_rescue, invalid_track_rendition_fallback,
video_player_lifecycle_contract, video_player_hls_authority_reactivation_contract.

### Measured swipe-to-first-frame on the second matrix (`e1c61852`)

See the percentile table in that run's section below; `focus_switch_ms` is the swipe-to-presented
time for a prepared reserve item, `startup_ms` the cold first item. Sample counts are in the
table; the method is nearest-rank over every `WARP_QOE` line the integration tests printed.

## Runs

## Run `20260901T212217Z-1c03551b85-none-target-contract`

- command: `make -s video-delivery-target-contract-test `
- commit: `1c03551b853ba403d6c3addbb2bd9447980724a2` (dirty_files=2)
- device: 
- exit: 0


## Run `20260901T212231Z-1c03551b85-22e0d933-physical-matrix`

- command: `/Applications/Xcode.app/Contents/Developer/usr/bin/make video-android-physical-tests ANDROID_PHYSICAL_SERIAL=22e0d933 `
- commit: `1c03551b853ba403d6c3addbb2bd9447980724a2` (dirty_files=2)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 2

| test file | last progress | result |
|---|---|---|
| integration_test/bandwidth_drop_video_test.dart | +1 | pass |
| integration_test/packet_loss_video_test.dart | +2 | pass |
| integration_test/high_rtt_video_test.dart | +3 | pass |
| integration_test/rapid_swipes_video_test.dart | +4 | pass |
| integration_test/held_response_video_test.dart | +5 | pass |
| integration_test/manifest_retry_video_test.dart | +6 | pass |
| integration_test/progressive_delivery_video_test.dart | +7 | pass |
| integration_test/warp_feed_progressive_loop_reopen_video_test.dart | +8 | pass |
| integration_test/warp_feed_playback_video_test.dart | +9 | pass |
| integration_test/warp_feed_visible_motion_video_test.dart | +10 | pass |
| integration_test/warp_feed_rapid_swipe_instrumentation_video_test.dart | +11 | pass |
| integration_test/warp_feed_adaptive_warm_back_video_test.dart | +11 -1 | FAIL |
| integration_test/warp_feed_startup_singleflight_video_test.dart | +11 -2 | FAIL |
| integration_test/warp_feed_player_verified_rescue_video_test.dart | +11 -2 | not run (stopped) |

### Swipe-to-first-frame samples (WARP_QOE lines, all test files in this run)

| metric | n | min | p50 | p95 | max | unit |
|---|---|---|---|---|---|---|
| startup_ms | 1 | 1543 | 1543 | 1543 | 1543 | ms |
| focus_switch_ms | 5 | 49 | 76 | 76 | 104 | ms |
| native_frame_ms | 6 | -6345 | -2131 | -574 | 1227 | ms |
| presented_ms | 6 | 49 | 76 | 104 | 1543 | ms |
| rust_ready_ms | 6 | -5899 | -986 | -285 | 1769 | ms |

Percentiles are nearest-rank over every sample printed by the integration tests in this
run; negative values mean the item was ready before the swipe (prepared reserve).

### WARP markers

```
WARP_LOOP ranged=2 coverage=293999/293999 duplicate=0 positions_ms=5003/102/608
WARP_PARALLEL revision=85 paths=/current.mp4,/next.mp4 byte_intervals_ms=1461-2101,1496-2101 bytes=65536,65536
WARP_CADENCE release_ms=183,216,107 focus_commit_ms=110,10,5
WARP_BURST target=5 ready=3 focus_intervals_ms=116,102 replenish_ms=5937
WARP_DECISION_HISTORY retained=67 first=106 latest=221
WARP_DECISION sequence=106 at=1788312580730 throughput_bps=1787320 planner_Bps=223415 slot_demand=false action=8 outcome=succeeded detail=null bytes=228463 elapsed_ms=1456 selected=fetch_range:transfer:97342262fb3f06a46896b0a6397cc1c5:https://cdd9da3445126d70c275b090fbf6cc56.invalid/0cc2c7860c56dfa785518a940ed4e192:65536-293999:target=null executed=97342262fb3f06a46896b0a6397cc1c5:https://cdd9da3445126d70c275b090fbf6cc56.invalid/0cc2c7860c56dfa785518a940ed4e192:65536-293999
WARP_DECISION sequence=133 at=1788312582250 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=9 outcome=succeeded detail=null bytes=65536 elapsed_ms=343 selected=prefix:transfer:51ce9d90d25a248ba66bba7754d34203:https://cdd9da3445126d70c275b090fbf6cc56.invalid/275ff592334ce8d1ad3a9cbd7e532089:0-65536:target=null executed=51ce9d90d25a248ba66bba7754d34203:https://cdd9da3445126d70c275b090fbf6cc56.invalid/275ff592334ce8d1ad3a9cbd7e532089:0-65536
WARP_DECISION sequence=142 at=1788312583063 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=10 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:03f157515a3efca903ccb776bdbd6c92:https://cdd9da3445126d70c275b090fbf6cc56.invalid/41a67055e513ed173e89748e6557976f:65536-293999:target=null executed=03f157515a3efca903ccb776bdbd6c92:https://cdd9da3445126d70c275b090fbf6cc56.invalid/41a67055e513ed173e89748e6557976f:65536-293999
WARP_DECISION sequence=158 at=1788312584015 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=159 at=1788312584098 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=160 at=1788312584182 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=161 at=1788312584244 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=162 at=1788312584323 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=163 at=1788312584365 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=164 at=1788312584450 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=165 at=1788312584491 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=166 at=1788312584571 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=167 at=1788312584666 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=168 at=1788312584739 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=169 at=1788312584774 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=170 at=1788312584822 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=171 at=1788312584889 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=172 at=1788312584943 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=173 at=1788312584986 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=174 at=1788312585038 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=175 at=1788312585111 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=176 at=1788312585151 throughput_bps=1609864 planner_Bps=201233 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=177 at=1788312585190 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=178 at=1788312585232 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=179 at=1788312585271 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=180 at=1788312585310 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=181 at=1788312585350 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=182 at=1788312585400 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=183 at=1788312585447 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=11 outcome=pending detail=null bytes=null elapsed_ms=null selected=prefix:transfer:076dfb4deb28792da0a524e58c4d09a2:https://cdd9da3445126d70c275b090fbf6cc56.invalid/d0d8c707aebf711ed99c6e3c4ebf3694:0-65536:target=null executed=076dfb4deb28792da0a524e58c4d09a2:https://cdd9da3445126d70c275b090fbf6cc56.invalid/d0d8c707aebf711ed99c6e3c4ebf3694:0-65536
WARP_DECISION sequence=184 at=1788312585495 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=185 at=1788312585534 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=186 at=1788312585606 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=187 at=1788312585664 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=188 at=1788312585754 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=189 at=1788312585792 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=190 at=1788312585845 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=191 at=1788312585889 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=192 at=1788312585927 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=193 at=1788312585965 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=194 at=1788312586012 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=195 at=1788312586089 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=196 at=1788312586147 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=197 at=1788312586185 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=198 at=1788312586226 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=199 at=1788312586270 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=200 at=1788312586313 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=201 at=1788312586378 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=12 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:51ce9d90d25a248ba66bba7754d34203:https://cdd9da3445126d70c275b090fbf6cc56.invalid/275ff592334ce8d1ad3a9cbd7e532089:65536-293999:target=null executed=51ce9d90d25a248ba66bba7754d34203:https://cdd9da3445126d70c275b090fbf6cc56.invalid/275ff592334ce8d1ad3a9cbd7e532089:65536-293999
WARP_DECISION sequence=202 at=1788312586438 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=203 at=1788312586498 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=204 at=1788312586543 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=205 at=1788312586621 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=206 at=1788312586697 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=207 at=1788312586790 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=208 at=1788312586865 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=209 at=1788312586938 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=210 at=1788312586981 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=211 at=1788312587025 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=212 at=1788312587075 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=213 at=1788312587138 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=214 at=1788312587174 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=215 at=1788312587230 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=216 at=1788312587267 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=217 at=1788312587344 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=218 at=1788312587406 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=219 at=1788312587462 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=220 at=1788312587495 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=221 at=1788312587548 throughput_bps=1496049 planner_Bps=187006 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_CADENCE release_ms=47,181,180 focus_commit_ms=7,7,7
WARP_REVERSE origin_before={fourth: (bytes: 293999, requests: 2), third: (bytes: 65536, requests: 1), next: (bytes: 65536, requests: 1)} origin_after={fourth: (bytes: 293999, requests: 2), third: (bytes: 293999, requests: 2), next: (bytes: 293999, requests: 2)}
WARP_REVERSE_REQUESTS seq=2:GET:/next.mp4:0-65536:served=65536:completed:time_us=1494952/1496648/2101627/2103800|seq=3:GET:/third.mp4:0-65536:served=65536:completed:time_us=2107250/2108443/2526992/2527337|seq=4:GET:/fourth.mp4:0-65536:served=65536:completed:time_us=3228973/3230719/3536101/3536465|seq=7:GET:/fourth.mp4:65536-293999:served=228463:completed:time_us=6504142/6505269/7917790/7918187|seq=13:GET:/third.mp4:65536-293999:served=228463:completed:time_us=16665858/16666820/18042644/18042935|seq=14:GET:/next.mp4:65536-293999:served=228463:completed:time_us=18264937/18265905/19595750/19596112
WARP_PARALLEL revision=89 paths=/current.mp4,/next.mp4 byte_intervals_ms=1459-2105,1509-2106 bytes=65536,65536
```


## Run `20260902T021700Z-6d01533772-22e0d933-targeted-eta-exhaustion`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_adaptive_warm_back_video_test.dart integration_test/warp_feed_player_verified_rescue_video_test.dart integration_test/warp_feed_origin_timeout_fallback_video_test.dart integration_test/progressive_delivery_video_test.dart -d 22e0d933 `
- commit: `6d01533772ea9c41253a1569a66a048fc9b3b789` (dirty_files=3)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_adaptive_warm_back_video_test.dart | +1 | pass |
| integration_test/warp_feed_player_verified_rescue_video_test.dart | +1 -1 | FAIL |
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +1 -2 | FAIL |
| integration_test/progressive_delivery_video_test.dart | +2 -2 | pass |

### Swipe-to-first-frame samples (WARP_QOE lines, all test files in this run)

| metric | n | min | p50 | p95 | max | unit |
|---|---|---|---|---|---|---|
| focus_switch_ms | 4 | 58 | 60 | 62 | 115 | ms |
| native_frame_ms | 4 | -3312 | -2567 | -2131 | -1749 | ms |
| presented_ms | 4 | 58 | 60 | 62 | 115 | ms |
| rust_ready_ms | 4 | -3057 | -1600 | -1044 | -556 | ms |

Percentiles are nearest-rank over every sample printed by the integration tests in this
run; negative values mean the item was ready before the swipe (prepared reserve).

### WARP markers

```
WARP_PARALLEL revision=84 paths=/current.mp4,/next.mp4 byte_intervals_ms=1444-2068,1480-2069 bytes=65536,65536
WARP_CADENCE release_ms=149,199,104 focus_commit_ms=21,10,9
WARP_WARM_TARGET id=21320c4dd9cf7f94c6ace493ff78e5c3f194ec07b333ce8a9530a8f7b53ee39a caption=WARP signed next media=http://127.0.0.1:40673/next.mp4
WARP_WARM_TARGET id=bfd46bc4a3ee94e6450b2e248bab59dce1a421326b186f973f4cc4d45e1ea992 caption=WARP signed third media=http://127.0.0.1:40673/third.mp4
WARP_WARM_TARGET id=cc060f82660c389e5da207d85a81376c55dd644255da85c83352102847609f9b caption=WARP signed fourth media=http://127.0.0.1:40673/fourth.mp4
WARP_BURST target=3 ready=3 focus_intervals_ms=187,103 replenish_ms=3904
WARP_DECISION_HISTORY retained=67 first=87 latest=220
WARP_DECISION sequence=87 at=1788316202637 throughput_bps=1595279 planner_Bps=199409 slot_demand=false action=8 outcome=cancelled detail=null bytes=32768 elapsed_ms=194 selected=fetch_range:transfer:ad8442b518e642796ca465b581b9654b:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/fc1e4f4c284ac59b6e31a068ca8b5b11:65536-293999:target=null executed=ad8442b518e642796ca465b581b9654b:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/fc1e4f4c284ac59b6e31a068ca8b5b11:65536-293999
WARP_DECISION sequence=127 at=1788316204200 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=9 outcome=succeeded detail=null bytes=228463 elapsed_ms=1470 selected=fetch_range:transfer:fa12b6bacda763a43feabcba25e74a45:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/826a35cedcb809dfdb2615ca75261de5:65536-293999:target=null executed=fa12b6bacda763a43feabcba25e74a45:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/826a35cedcb809dfdb2615ca75261de5:65536-293999
WARP_DECISION sequence=133 at=1788316204460 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=10 outcome=succeeded detail=null bytes=65536 elapsed_ms=335 selected=prefix:transfer:4df42656013e13af1f49a5167d23ee3b:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/bf669c22cd66d3261c348c900fc00767:0-65536:target=null executed=4df42656013e13af1f49a5167d23ee3b:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/bf669c22cd66d3261c348c900fc00767:0-65536
WARP_DECISION sequence=157 at=1788316205743 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=158 at=1788316205829 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=159 at=1788316205877 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=160 at=1788316205931 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=161 at=1788316205960 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=162 at=1788316206027 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=163 at=1788316206056 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=11 outcome=pending detail=null bytes=null elapsed_ms=null selected=prefix:transfer:15246b2f23b61448881483ec0c2d427c:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/c2c8e3b984a9bdf01f61d7b62213cc84:0-65536:target=null executed=15246b2f23b61448881483ec0c2d427c:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/c2c8e3b984a9bdf01f61d7b62213cc84:0-65536
WARP_DECISION sequence=164 at=1788316206119 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=165 at=1788316206167 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=166 at=1788316206227 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=167 at=1788316206318 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=168 at=1788316206381 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=169 at=1788316206487 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=170 at=1788316206559 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=171 at=1788316206588 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=172 at=1788316206617 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=173 at=1788316206647 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=174 at=1788316206682 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=175 at=1788316206737 throughput_bps=1582226 planner_Bps=197778 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=176 at=1788316206772 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=177 at=1788316206850 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=178 at=1788316206896 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=179 at=1788316206954 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=180 at=1788316206996 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=181 at=1788316207039 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=182 at=1788316207068 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=183 at=1788316207100 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=184 at=1788316207128 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=185 at=1788316207159 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=186 at=1788316207200 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=187 at=1788316207232 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=188 at=1788316207265 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=189 at=1788316207307 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=190 at=1788316207344 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=191 at=1788316207371 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=192 at=1788316207398 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=193 at=1788316207423 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=194 at=1788316207458 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=195 at=1788316207482 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=196 at=1788316207508 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=12 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:a9a2d0fd32cb3272437a3e85425b3099:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/9396ad6ea27cf99b41867027e94dc0ba:65536-293999:target=null executed=a9a2d0fd32cb3272437a3e85425b3099:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/9396ad6ea27cf99b41867027e94dc0ba:65536-293999
WARP_DECISION sequence=197 at=1788316207538 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=198 at=1788316207574 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=199 at=1788316207621 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=200 at=1788316207644 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=201 at=1788316207679 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=202 at=1788316207708 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=203 at=1788316207734 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=204 at=1788316207767 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=205 at=1788316207813 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=206 at=1788316207834 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=207 at=1788316207861 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=208 at=1788316207886 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=209 at=1788316207930 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=210 at=1788316207952 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=211 at=1788316207979 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=212 at=1788316208013 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=213 at=1788316208055 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=214 at=1788316208080 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=215 at=1788316208126 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=13 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:4df42656013e13af1f49a5167d23ee3b:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/bf669c22cd66d3261c348c900fc00767:65536-293999:target=null executed=4df42656013e13af1f49a5167d23ee3b:https://c2bd7c3b6bbe64d58d445185652d79c1.invalid/bf669c22cd66d3261c348c900fc00767:65536-293999
WARP_DECISION sequence=216 at=1788316208170 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=217 at=1788316208207 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=218 at=1788316208256 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=219 at=1788316208291 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=220 at=1788316208360 throughput_bps=1582306 planner_Bps=197788 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_CADENCE release_ms=87,225,84 focus_commit_ms=6,10,10
WARP_DECISION_HISTORY retained=64 first=407 latest=470
WARP_DECISION sequence=407 at=1788316313640 throughput_bps=1765594 planner_Bps=220699 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=408 at=1788316313677 throughput_bps=1765594 planner_Bps=220699 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=409 at=1788316313776 throughput_bps=1765594 planner_Bps=220699 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=410 at=1788316313877 throughput_bps=1765594 planner_Bps=220699 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
```


## Run `20260902T023442Z-ba07a36c24-22e0d933-targeted-rerun`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_player_verified_rescue_video_test.dart integration_test/warp_feed_origin_timeout_fallback_video_test.dart -d 22e0d933 `
- commit: `ba07a36c248c53fff87e2efde6501426d233c651` (dirty_files=4)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_player_verified_rescue_video_test.dart | +1 | pass |
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +1 -1 | FAIL |

### WARP markers

```
WARP_DECISION_HISTORY retained=64 first=405 latest=468
WARP_DECISION sequence=405 at=1788316586620 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=406 at=1788316586720 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=407 at=1788316586819 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=408 at=1788316586833 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=409 at=1788316586919 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=410 at=1788316586974 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=411 at=1788316587020 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=412 at=1788316587119 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=413 at=1788316587220 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=414 at=1788316587320 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=415 at=1788316587334 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=416 at=1788316587419 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=417 at=1788316587520 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=418 at=1788316587619 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=419 at=1788316587720 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=420 at=1788316587820 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=421 at=1788316587836 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=422 at=1788316587919 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=423 at=1788316587974 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=424 at=1788316588020 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=425 at=1788316588120 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=426 at=1788316588220 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=427 at=1788316588238 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=428 at=1788316588320 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=429 at=1788316588334 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=430 at=1788316588420 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=431 at=1788316588520 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=432 at=1788316588620 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=433 at=1788316588720 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=434 at=1788316588820 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=435 at=1788316588834 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=436 at=1788316588920 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=437 at=1788316588980 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=438 at=1788316589020 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=439 at=1788316589120 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=440 at=1788316589220 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=441 at=1788316589266 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=442 at=1788316589321 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=443 at=1788316589336 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=444 at=1788316589419 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=445 at=1788316589521 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=446 at=1788316589620 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=447 at=1788316589720 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=448 at=1788316589819 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=449 at=1788316589833 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=450 at=1788316589919 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=451 at=1788316589976 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=452 at=1788316590020 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=453 at=1788316590119 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=454 at=1788316590219 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=455 at=1788316590319 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=456 at=1788316590336 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=457 at=1788316590351 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=458 at=1788316590419 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=459 at=1788316590522 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=460 at=1788316590619 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=461 at=1788316590719 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=462 at=1788316590820 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=463 at=1788316590833 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=464 at=1788316590921 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=465 at=1788316590975 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=466 at=1788316591021 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=467 at=1788316591121 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=468 at=1788316591222 throughput_bps=2465003 planner_Bps=308125 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
```


## Run `20260902T023705Z-4d983a4240-22e0d933-bisect-origin-timeout-pre-slices`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_origin_timeout_fallback_video_test.dart -d 22e0d933 `
- commit: `4d983a42404b007096aa801527e413b31fe3425a` (dirty_files=3)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +0 -1 | FAIL |


## Run `20260902T025115Z-301db545aa-22e0d933-origin-timeout-quantized-eta`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_origin_timeout_fallback_video_test.dart -d 22e0d933 `
- commit: `301db545aa80855853e959a1455194cae16d8188` (dirty_files=8)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +0 -1 | FAIL |


## Run `20260902T030112Z-e1c61852b9-22e0d933-physical-matrix`

- command: `/Applications/Xcode.app/Contents/Developer/usr/bin/make video-android-physical-tests ANDROID_PHYSICAL_SERIAL=22e0d933 `
- commit: `e1c61852b913cd4113757f17fa9cd75f6690178c` (dirty_files=6)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 2

| test file | last progress | result |
|---|---|---|
| integration_test/bandwidth_drop_video_test.dart | +1 | pass |
| integration_test/packet_loss_video_test.dart | +2 | pass |
| integration_test/high_rtt_video_test.dart | +3 | pass |
| integration_test/rapid_swipes_video_test.dart | +4 | pass |
| integration_test/held_response_video_test.dart | +5 | pass |
| integration_test/manifest_retry_video_test.dart | +6 | pass |
| integration_test/progressive_delivery_video_test.dart | +7 | pass |
| integration_test/warp_feed_progressive_loop_reopen_video_test.dart | +8 | pass |
| integration_test/warp_feed_playback_video_test.dart | +9 | pass |
| integration_test/warp_feed_visible_motion_video_test.dart | +10 | pass |
| integration_test/warp_feed_rapid_swipe_instrumentation_video_test.dart | +11 | pass |
| integration_test/warp_feed_adaptive_warm_back_video_test.dart | +11 -1 | FAIL |
| integration_test/warp_feed_startup_singleflight_video_test.dart | +12 -1 | pass |
| integration_test/warp_feed_player_verified_rescue_video_test.dart | +12 -2 | FAIL |
| integration_test/warp_feed_mixed_hls_readiness_video_test.dart | +13 -2 | pass |
| integration_test/warp_feed_bandwidth_recovery_video_test.dart | +14 -2 | pass |
| integration_test/warp_feed_ignored_range_video_test.dart | +15 -2 | pass |
| integration_test/warp_feed_malformed_range_rescue_video_test.dart | +16 -2 | pass |
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +16 -3 | FAIL |
| integration_test/warp_feed_long_session_boundedness_video_test.dart | +16 -4 | FAIL |
| integration_test/warp_feed_cache_pressure_video_test.dart | +16 -5 | FAIL |
| integration_test/warp_feed_invalid_track_rendition_fallback_video_test.dart | +17 -5 | pass |
| integration_test/warp_feed_unsupported_hls_rescue_video_test.dart | +17 -6 | FAIL |
| integration_test/warp_feed_stale_validator_rotation_video_test.dart | +17 -7 | FAIL |
| integration_test/video_player_lifecycle_contract_test.dart | +18 -7 | pass |
| integration_test/video_player_hls_authority_reactivation_contract_test.dart | +19 -7 | pass |

### Swipe-to-first-frame samples (WARP_QOE lines, all test files in this run)

| metric | n | min | p50 | p95 | max | unit |
|---|---|---|---|---|---|---|
| startup_ms | 3 | 1116 | 1249 | 1249 | 1317 | ms |
| focus_switch_ms | 5 | 61 | 88 | 90 | 160 | ms |
| native_frame_ms | 8 | -5603 | -1225 | 1091 | 1114 | ms |
| presented_ms | 8 | 61 | 90 | 1249 | 1317 | ms |
| rust_ready_ms | 6 | -5109 | -2269 | -362 | 1652 | ms |

Percentiles are nearest-rank over every sample printed by the integration tests in this
run; negative values mean the item was ready before the swipe (prepared reserve).

### WARP markers

```
WARP_LOOP ranged=2 coverage=293999/293999 duplicate=0 positions_ms=5000/94/599
WARP_PARALLEL revision=103 paths=/current.mp4,/next.mp4 byte_intervals_ms=1581-2313,1584-2313 bytes=65536,65536
WARP_CADENCE release_ms=209,285,106 focus_commit_ms=73,14,15
WARP_BURST target=5 ready=3 focus_intervals_ms=226,107 replenish_ms=6100
WARP_DECISION_HISTORY retained=67 first=132 latest=242
WARP_DECISION sequence=132 at=1788318435479 throughput_bps=1527005 planner_Bps=190875 slot_demand=false action=9 outcome=succeeded detail=null bytes=65536 elapsed_ms=354 selected=prefix:transfer:44ca3ddf0a892cb38cdd758ac3df9a94:https://074c98e281271271e3be19710af4086e.invalid/3685e79d586f167616e3b0e13b5e7f89:0-65536:target=null executed=44ca3ddf0a892cb38cdd758ac3df9a94:https://074c98e281271271e3be19710af4086e.invalid/3685e79d586f167616e3b0e13b5e7f89:0-65536
WARP_DECISION sequence=152 at=1788318436538 throughput_bps=1527005 planner_Bps=190875 slot_demand=false action=10 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:9982416901ce74a4453d9666dc633bcc:https://074c98e281271271e3be19710af4086e.invalid/3df0b82ffc820b7cb6007a3310c35108:65536-293999:target=null executed=9982416901ce74a4453d9666dc633bcc:https://074c98e281271271e3be19710af4086e.invalid/3df0b82ffc820b7cb6007a3310c35108:65536-293999
WARP_DECISION sequence=172 at=1788318437689 throughput_bps=1527005 planner_Bps=190875 slot_demand=false action=11 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:aebb8c33dce246bc604b751cc6d4052f:https://074c98e281271271e3be19710af4086e.invalid/ff1aa06ed1cd82926cb60c3edc842a5a:65536-293999:target=null executed=aebb8c33dce246bc604b751cc6d4052f:https://074c98e281271271e3be19710af4086e.invalid/ff1aa06ed1cd82926cb60c3edc842a5a:65536-293999
WARP_DECISION sequence=179 at=1788318438447 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=180 at=1788318438563 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=181 at=1788318438646 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=182 at=1788318438747 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=183 at=1788318438851 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=184 at=1788318438964 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=185 at=1788318439050 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=186 at=1788318439160 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=187 at=1788318439302 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=188 at=1788318439401 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=189 at=1788318439454 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=190 at=1788318439496 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=191 at=1788318439533 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=192 at=1788318439573 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=193 at=1788318439606 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=194 at=1788318439642 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=195 at=1788318439685 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=196 at=1788318439723 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=197 at=1788318439765 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=198 at=1788318439801 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=199 at=1788318439837 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=200 at=1788318439877 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=201 at=1788318439915 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=202 at=1788318439952 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=203 at=1788318439992 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=12 outcome=pending detail=null bytes=null elapsed_ms=null selected=prefix:transfer:ef37f24e5ff88db7d8d3af77b519ed59:https://074c98e281271271e3be19710af4086e.invalid/6741301e7255f2fc977b63bcc0a5830e:0-65536:target=null executed=ef37f24e5ff88db7d8d3af77b519ed59:https://074c98e281271271e3be19710af4086e.invalid/6741301e7255f2fc977b63bcc0a5830e:0-65536
WARP_DECISION sequence=204 at=1788318440032 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=205 at=1788318440069 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=206 at=1788318440162 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=207 at=1788318440259 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=208 at=1788318440300 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=209 at=1788318440343 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=210 at=1788318440473 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=211 at=1788318440514 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=212 at=1788318440558 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=213 at=1788318440605 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=214 at=1788318440646 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=215 at=1788318440690 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=216 at=1788318440754 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=217 at=1788318440817 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=218 at=1788318440855 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=219 at=1788318440894 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=220 at=1788318440930 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=221 at=1788318441002 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=222 at=1788318441046 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=223 at=1788318441112 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=224 at=1788318441210 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=225 at=1788318441271 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=226 at=1788318441362 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=227 at=1788318441456 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=228 at=1788318441520 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=229 at=1788318441605 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=230 at=1788318441703 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=231 at=1788318441801 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=232 at=1788318441856 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=233 at=1788318441900 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=234 at=1788318441931 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=235 at=1788318441964 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=236 at=1788318442005 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=237 at=1788318442036 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=238 at=1788318442066 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=239 at=1788318442098 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=240 at=1788318442132 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=241 at=1788318442163 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=242 at=1788318442199 throughput_bps=1498292 planner_Bps=187286 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_CADENCE release_ms=95,250,133 focus_commit_ms=17,13,11
WARP_REVERSE origin_before={fourth: (bytes: 293999, requests: 2), third: (bytes: 65536, requests: 1), next: (bytes: 293999, requests: 2)} origin_after={fourth: (bytes: 293999, requests: 2), third: (bytes: 293999, requests: 2), next: (bytes: 293999, requests: 2)}
WARP_REVERSE_REQUESTS seq=2:GET:/next.mp4:0-65536:served=65536:completed:time_us=1582822/1584658/2313835/2315515|seq=3:GET:/third.mp4:0-65536:served=65536:completed:time_us=2584671/2586211/2984039/2984643|seq=4:GET:/next.mp4:65536-293999:served=228463:completed:time_us=2614589/2615948/4204506/4204999|seq=6:GET:/fourth.mp4:0-65536:served=65536:completed:time_us=6004249/6007639/6329718/6330200|seq=9:GET:/fourth.mp4:65536-293999:served=228463:completed:time_us=10266390/10267674/11854894/11855750|seq=13:GET:/third.mp4:65536-293999:served=228463:completed:time_us=18171454/18172656/19698000/19698428
WARP_PARALLEL revision=95 paths=/current.mp4,/next.mp4 byte_intervals_ms=1995-2837,1999-2837 bytes=65536,65536
WARP_CADENCE release_ms=221,254,121 focus_commit_ms=35,17,15
WARP_WARM_TARGET id=fe9d9e00c6abe254a73b98a39a0b601bcd7f38007bc97493c10009de90e3e6bd caption=WARP signed next media=http://127.0.0.1:37587/next.mp4
WARP_WARM_TARGET id=d05fab46c272378fe863f6f971ae27b2b2f419d619a9674986862e257a94b032 caption=WARP signed third media=http://127.0.0.1:37587/third.mp4
WARP_DECISION_HISTORY retained=49 first=1 latest=49
```

### First-frame lines

```
WARP_HLS_CLEANUP delivery=9fe1f9d9f5440aa3b682cf48c6bb5611355a84226387242fd8e110ed8d912249 authority=9fe1f9d9f5440aa3b682cf48c6bb5611355a84226387242fd8e110ed8d9
```

## Run `20260902T032053Z-e1c61852b9-22e0d933-isolation-warm-back-and-rescue`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_adaptive_warm_back_video_test.dart integration_test/warp_feed_player_verified_rescue_video_test.dart -d 22e0d933 `
- commit: `e1c61852b913cd4113757f17fa9cd75f6690178c` (dirty_files=7)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_adaptive_warm_back_video_test.dart | +0 -1 | FAIL |
| integration_test/warp_feed_player_verified_rescue_video_test.dart | +1 -1 | pass |

### WARP markers

```
WARP_PARALLEL revision=93 paths=/current.mp4,/next.mp4 byte_intervals_ms=1736-2528,1741-2528 bytes=65536,65536
WARP_CADENCE release_ms=204,218,175 focus_commit_ms=32,13,8
WARP_WARM_TARGET id=bd8617ffc19e089f1037d63a33dad59bf978f6d969c5bbd55f535f87f39e1101 caption=WARP signed next media=http://127.0.0.1:38093/next.mp4
```


## Run `20260902T032330Z-e1c61852b9-22e0d933-warm-back-x2-quantized`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_adaptive_warm_back_video_test.dart integration_test/warp_feed_adaptive_warm_back_video_test.dart -d 22e0d933 `
- commit: `e1c61852b913cd4113757f17fa9cd75f6690178c` (dirty_files=8)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 0

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_adaptive_warm_back_video_test.dart | +1 | pass |

### Swipe-to-first-frame samples (WARP_QOE lines, all test files in this run)

| metric | n | min | p50 | p95 | max | unit |
|---|---|---|---|---|---|---|
| focus_switch_ms | 4 | 63 | 65 | 89 | 137 | ms |
| native_frame_ms | 4 | -4642 | -4352 | -4017 | -1102 | ms |
| presented_ms | 4 | 63 | 65 | 89 | 137 | ms |
| rust_ready_ms | 4 | -3403 | -3239 | -2547 | -625 | ms |

Percentiles are nearest-rank over every sample printed by the integration tests in this
run; negative values mean the item was ready before the swipe (prepared reserve).

### WARP markers

```
WARP_PARALLEL revision=103 paths=/current.mp4,/next.mp4 byte_intervals_ms=1926-2757,1931-2758 bytes=65536,65536
WARP_CADENCE release_ms=236,217,129 focus_commit_ms=31,10,10
WARP_WARM_TARGET id=95d3a5468684273bcc8145d4a7f22255f613fbce703b2c248ce838105500b926 caption=WARP signed next media=http://127.0.0.1:41291/next.mp4
WARP_WARM_TARGET id=b7392ce86c3e5a18f06601fd482ecb265445880e13c51fc9d6cf65a7fa22998f caption=WARP signed third media=http://127.0.0.1:41291/third.mp4
WARP_WARM_TARGET id=7c2024635770dd49ed33f1f74c9bd350c7f1459af403e6476d67cf9c47ad01a8 caption=WARP signed fourth media=http://127.0.0.1:41291/fourth.mp4
WARP_BURST target=5 ready=3 focus_intervals_ms=196,128 replenish_ms=4506
WARP_DECISION_HISTORY retained=67 first=106 latest=224
WARP_DECISION sequence=106 at=1788319448558 throughput_bps=1982766 planner_Bps=247845 slot_demand=false action=8 outcome=succeeded detail=null bytes=65536 elapsed_ms=478 selected=prefix:transfer:5b311184322448c1dff23f6b3746a689:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/577c387e929ad84a6b611fe9680b1e18:0-65536:target=null executed=5b311184322448c1dff23f6b3746a689:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/577c387e929ad84a6b611fe9680b1e18:0-65536
WARP_DECISION sequence=122 at=1788319449397 throughput_bps=1982766 planner_Bps=247845 slot_demand=false action=9 outcome=succeeded detail=null bytes=65536 elapsed_ms=377 selected=prefix:transfer:0bcb55f2d96ba3bb66a175c62af8f2af:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/0073682270c563eef2737efd080639f8:0-65536:target=null executed=0bcb55f2d96ba3bb66a175c62af8f2af:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/0073682270c563eef2737efd080639f8:0-65536
WARP_DECISION sequence=144 at=1788319450450 throughput_bps=1982766 planner_Bps=247845 slot_demand=false action=10 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:40915a9d4746f9d7d2abdee2520daa5a:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/609d3fd5e172e7ed1f7e9dc9e900f241:65536-293999:target=null executed=40915a9d4746f9d7d2abdee2520daa5a:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/609d3fd5e172e7ed1f7e9dc9e900f241:65536-293999
WARP_DECISION sequence=161 at=1788319451452 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=162 at=1788319451532 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=163 at=1788319451569 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=164 at=1788319451651 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=165 at=1788319451743 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=166 at=1788319451794 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=167 at=1788319451861 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=168 at=1788319451973 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=169 at=1788319452122 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=170 at=1788319452222 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=171 at=1788319452312 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=172 at=1788319452375 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=173 at=1788319452434 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=174 at=1788319452520 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=175 at=1788319452612 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=176 at=1788319452703 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=177 at=1788319452796 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=11 outcome=pending detail=null bytes=null elapsed_ms=null selected=prefix:transfer:5c36dba2909fee0ef756389c329c10a1:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/893eeedb890584c55b91055b89b13866:0-65536:target=null executed=5c36dba2909fee0ef756389c329c10a1:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/893eeedb890584c55b91055b89b13866:0-65536
WARP_DECISION sequence=178 at=1788319452886 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=179 at=1788319452978 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=180 at=1788319453054 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=181 at=1788319453125 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=182 at=1788319453176 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=183 at=1788319453210 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=184 at=1788319453249 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=185 at=1788319453311 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=186 at=1788319453347 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=187 at=1788319453383 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=188 at=1788319453417 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=189 at=1788319453456 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=190 at=1788319453503 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=191 at=1788319453561 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=192 at=1788319453600 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=193 at=1788319453642 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=194 at=1788319453698 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=195 at=1788319453742 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=196 at=1788319453788 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=197 at=1788319453827 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=198 at=1788319453865 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=199 at=1788319453909 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=200 at=1788319453950 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=201 at=1788319453980 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=202 at=1788319454007 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=203 at=1788319454078 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=204 at=1788319454126 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=205 at=1788319454160 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=12 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:5b311184322448c1dff23f6b3746a689:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/577c387e929ad84a6b611fe9680b1e18:65536-293999:target=null executed=5b311184322448c1dff23f6b3746a689:https://98e2629c8dccf02d1ae827c5d1fa99cb.invalid/577c387e929ad84a6b611fe9680b1e18:65536-293999
WARP_DECISION sequence=206 at=1788319454216 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=207 at=1788319454267 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=208 at=1788319454298 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=209 at=1788319454340 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=210 at=1788319454404 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=211 at=1788319454452 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=212 at=1788319454504 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=213 at=1788319454536 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=214 at=1788319454593 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=215 at=1788319454625 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=216 at=1788319454665 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=217 at=1788319454715 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=218 at=1788319454752 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=219 at=1788319454809 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=220 at=1788319454848 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=221 at=1788319454916 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=222 at=1788319454951 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=223 at=1788319454992 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=224 at=1788319455090 throughput_bps=1651871 planner_Bps=206483 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_CADENCE release_ms=49,243,269 focus_commit_ms=8,7,6
```


## Run `20260902T032418Z-6d01533772-22e0d933-warm-back-x2-unquantized`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_adaptive_warm_back_video_test.dart integration_test/warp_feed_adaptive_warm_back_video_test.dart -d 22e0d933 `
- commit: `6d01533772ea9c41253a1569a66a048fc9b3b789` (dirty_files=8)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 0

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_adaptive_warm_back_video_test.dart | +1 | pass |

### Swipe-to-first-frame samples (WARP_QOE lines, all test files in this run)

| metric | n | min | p50 | p95 | max | unit |
|---|---|---|---|---|---|---|
| focus_switch_ms | 4 | 68 | 75 | 79 | 86 | ms |
| native_frame_ms | 4 | -5682 | -2112 | -1734 | -1088 | ms |
| presented_ms | 4 | 68 | 75 | 79 | 86 | ms |
| rust_ready_ms | 4 | -5233 | -1716 | -1210 | -570 | ms |

Percentiles are nearest-rank over every sample printed by the integration tests in this
run; negative values mean the item was ready before the swipe (prepared reserve).

### WARP markers

```
WARP_PARALLEL revision=81 paths=/current.mp4,/next.mp4 byte_intervals_ms=1542-2222,1555-2222 bytes=65536,65536
WARP_CADENCE release_ms=181,167,93 focus_commit_ms=28,16,20
WARP_WARM_TARGET id=4bf27d985a1e0212aa70e12dde9b78e230d4d803e7cb5b95a41d3cf8e7664410 caption=WARP signed next media=http://127.0.0.1:40511/next.mp4
WARP_WARM_TARGET id=69139779a0593cf63dcd1d65d343987bdbce36c2ba9ce479ee3245f8dfaf9519 caption=WARP signed third media=http://127.0.0.1:40511/third.mp4
WARP_WARM_TARGET id=bc40e492eee55a6784257e76bc02dca48dfa1f79115a79361dbcc3bb2efec54e caption=WARP signed fourth media=http://127.0.0.1:40511/fourth.mp4
WARP_BURST target=3 ready=3 focus_intervals_ms=155,97 replenish_ms=5591
WARP_DECISION_HISTORY retained=67 first=109 latest=227
WARP_DECISION sequence=109 at=1788319911765 throughput_bps=1581812 planner_Bps=197726 slot_demand=false action=8 outcome=succeeded detail=null bytes=228463 elapsed_ms=1566 selected=fetch_range:transfer:13930f16175131fc1ce1a6238c1d8ad8:https://b24f05377f1dce8f648df512b1881648.invalid/5a0c1233ef5a068cbb5581586dd90844:65536-293999:target=null executed=13930f16175131fc1ce1a6238c1d8ad8:https://b24f05377f1dce8f648df512b1881648.invalid/5a0c1233ef5a068cbb5581586dd90844:65536-293999
WARP_DECISION sequence=129 at=1788319912748 throughput_bps=1547975 planner_Bps=193496 slot_demand=false action=9 outcome=succeeded detail=null bytes=65536 elapsed_ms=371 selected=prefix:transfer:4725db185a070884b586b2e19181e13d:https://b24f05377f1dce8f648df512b1881648.invalid/49f879f0e9ef736218309ace847c5e22:0-65536:target=null executed=4725db185a070884b586b2e19181e13d:https://b24f05377f1dce8f648df512b1881648.invalid/49f879f0e9ef736218309ace847c5e22:0-65536
WARP_DECISION sequence=142 at=1788319913591 throughput_bps=1547975 planner_Bps=193496 slot_demand=false action=10 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:9d192b76de9a22c4e87fa9ec0c19c841:https://b24f05377f1dce8f648df512b1881648.invalid/c1bb9b7b725fdcc8b144998f992137a7:65536-293999:target=null executed=9d192b76de9a22c4e87fa9ec0c19c841:https://b24f05377f1dce8f648df512b1881648.invalid/c1bb9b7b725fdcc8b144998f992137a7:65536-293999
WARP_DECISION sequence=164 at=1788319914921 throughput_bps=1547975 planner_Bps=193496 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=165 at=1788319914990 throughput_bps=1547975 planner_Bps=193496 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=166 at=1788319915042 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=167 at=1788319915118 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=168 at=1788319915255 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=169 at=1788319915301 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=170 at=1788319915364 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=171 at=1788319915398 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=172 at=1788319915445 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=173 at=1788319915502 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=174 at=1788319915543 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=175 at=1788319915590 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=176 at=1788319915626 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=177 at=1788319915662 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=178 at=1788319915696 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=179 at=1788319915727 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=180 at=1788319915767 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=181 at=1788319915829 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=182 at=1788319915861 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=11 outcome=pending detail=null bytes=null elapsed_ms=null selected=prefix:transfer:eb0b03c4a6e6559e9f7c474390ef6a4e:https://b24f05377f1dce8f648df512b1881648.invalid/3cde1bf2978b136f779720a53ae97e58:0-65536:target=null executed=eb0b03c4a6e6559e9f7c474390ef6a4e:https://b24f05377f1dce8f648df512b1881648.invalid/3cde1bf2978b136f779720a53ae97e58:0-65536
WARP_DECISION sequence=183 at=1788319915897 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=184 at=1788319915932 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=185 at=1788319916009 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=186 at=1788319916079 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=187 at=1788319916109 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=188 at=1788319916151 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=189 at=1788319916185 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=190 at=1788319916216 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=191 at=1788319916251 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=192 at=1788319916310 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=193 at=1788319916338 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=194 at=1788319916376 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=195 at=1788319916407 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=196 at=1788319916440 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=197 at=1788319916473 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=198 at=1788319916543 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=199 at=1788319916606 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=200 at=1788319916651 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=201 at=1788319916698 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=202 at=1788319916741 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=203 at=1788319916803 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=204 at=1788319916844 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=205 at=1788319916889 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=206 at=1788319916946 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=207 at=1788319917005 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=12 outcome=pending detail=null bytes=null elapsed_ms=null selected=fetch_range:transfer:4725db185a070884b586b2e19181e13d:https://b24f05377f1dce8f648df512b1881648.invalid/49f879f0e9ef736218309ace847c5e22:65536-293999:target=null executed=4725db185a070884b586b2e19181e13d:https://b24f05377f1dce8f648df512b1881648.invalid/49f879f0e9ef736218309ace847c5e22:65536-293999
WARP_DECISION sequence=208 at=1788319917078 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=209 at=1788319917154 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=210 at=1788319917237 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=211 at=1788319917324 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=212 at=1788319917383 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=213 at=1788319917447 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=214 at=1788319917507 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=215 at=1788319917564 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=216 at=1788319917608 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=217 at=1788319917647 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=218 at=1788319917681 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=219 at=1788319917712 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=220 at=1788319917747 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=221 at=1788319917784 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=222 at=1788319917818 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=223 at=1788319917882 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=224 at=1788319917923 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=225 at=1788319917993 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=226 at=1788319918030 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=227 at=1788319918091 throughput_bps=1477382 planner_Bps=184672 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_CADENCE release_ms=90,279,139 focus_commit_ms=9,12,7
```


## Run `20260902T033226Z-e1c61852b9-22e0d933-offline-restart`

- command: `make video-android-offline-restart ANDROID_PHYSICAL_SERIAL=22e0d933 `
- commit: `e1c61852b913cd4113757f17fa9cd75f6690178c` (dirty_files=10)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 0

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_offline_seed_video_test.dart | +0 | not run (stopped) |
| integration_test/warp_feed_offline_restore_video_test.dart | +1 | pass |

### WARP markers

```
WARP_OFFLINE_SEED event=f684d58536eccca3255c677eafc91f2afe2dd347549c9749f12fcc2f0ef8b0ba origin=http://127.0.0.1:40653 frameUs=948039 unique=293999/293999 duplicate=0 ranges=2
WARP_OFFLINE_RESTORE event=f684d58536eccca3255c677eafc91f2afe2dd347549c9749f12fcc2f0ef8b0ba origin=http://127.0.0.1:40653 frameUs=1185936 originRequests=0 networkRequests=0 attempts=1 released=1
```


## Run `20260902T034917Z-e1c61852b9-22e0d933-lifecycle`

- command: `make video-android-lifecycle ANDROID_PHYSICAL_SERIAL=22e0d933 `
- commit: `e1c61852b913cd4113757f17fa9cd75f6690178c` (dirty_files=11)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 2

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_android_lifecycle_video_test.dart | +0 -1 | FAIL |

### WARP markers

```
WARP_ANDROID_FOREGROUND_DIAGNOSTICS
WARP_ANDROID_LIFECYCLE_RESUMED.
WARP_ANDROID_LIFECYCLE_READY session=1
WARP_ANDROID_LIFECYCLE_BACKGROUND states=resumed|inactive|hidden|paused
```


## Run `20260902T051208Z-314742d3db-22e0d933-origin-timeout-peer-close`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_origin_timeout_fallback_video_test.dart -d 22e0d933 `
- commit: `314742d3db66eb081ef72e8cc46d7e17ba9a225d` (dirty_files=7)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +0 -1 | FAIL |

### WARP markers

```
WARP_ORIGIN_TIMEOUT peer_closed_ms=16599 fallback_started_ms=16663 fallback_finished_ms=17324
```


## Run `20260902T051335Z-314742d3db-22e0d933-gate-consumers-after-peer-close`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_player_verified_rescue_video_test.dart integration_test/warp_feed_long_session_boundedness_video_test.dart integration_test/warp_feed_mixed_hls_readiness_video_test.dart -d 22e0d933 `
- commit: `314742d3db66eb081ef72e8cc46d7e17ba9a225d` (dirty_files=8)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_player_verified_rescue_video_test.dart | +0 -1 | FAIL |
| integration_test/warp_feed_long_session_boundedness_video_test.dart | +0 -2 | FAIL |
| integration_test/warp_feed_mixed_hls_readiness_video_test.dart | +1 -2 | pass |

### WARP markers

```
WARP_DECISION_HISTORY retained=64 first=3 latest=307
WARP_DECISION sequence=3 at=1788326043906 throughput_bps=33554432 planner_Bps=4194304 slot_demand=false action=2 outcome=failed detail=Transient bytes=null elapsed_ms=15235 selected=prefix:transfer:2670cd137d5e0a0d2a1403eb09f5fb75:https://29f08b56b3eeeabc6da441d112a0a90d.invalid/b8675dd3640f948a39a6d65c76e79bf1:0-65536:target=null executed=2670cd137d5e0a0d2a1403eb09f5fb75:https://29f08b56b3eeeabc6da441d112a0a90d.invalid/b8675dd3640f948a39a6d65c76e79bf1:0-65536
WARP_DECISION sequence=245 at=1788326056129 throughput_bps=1125965 planner_Bps=140745 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=246 at=1788326056229 throughput_bps=1125965 planner_Bps=140745 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=247 at=1788326056251 throughput_bps=1125965 planner_Bps=140745 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=248 at=1788326056317 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=249 at=1788326056336 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=250 at=1788326056429 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=251 at=1788326056529 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=252 at=1788326056629 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=253 at=1788326056729 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=254 at=1788326056753 throughput_bps=1050271 planner_Bps=131283 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=255 at=1788326056819 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=256 at=1788326056836 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=257 at=1788326056929 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=258 at=1788326057029 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=259 at=1788326057129 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=260 at=1788326057149 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=261 at=1788326057229 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=262 at=1788326057251 throughput_bps=979666 planner_Bps=122458 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=263 at=1788326057320 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=264 at=1788326057338 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=265 at=1788326057429 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=266 at=1788326057529 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=267 at=1788326057629 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=268 at=1788326057729 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=269 at=1788326057750 throughput_bps=913934 planner_Bps=114241 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=270 at=1788326057822 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=271 at=1788326057840 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=272 at=1788326057859 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=273 at=1788326057929 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=274 at=1788326058029 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=275 at=1788326058130 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=276 at=1788326058230 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=277 at=1788326058251 throughput_bps=852494 planner_Bps=106561 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=278 at=1788326058323 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=279 at=1788326058343 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=280 at=1788326058364 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=281 at=1788326058430 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=282 at=1788326058530 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=283 at=1788326058630 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=284 at=1788326058730 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=285 at=1788326058750 throughput_bps=795295 planner_Bps=99411 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=286 at=1788326058824 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=287 at=1788326058844 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=288 at=1788326058930 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=289 at=1788326059030 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=290 at=1788326059130 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=291 at=1788326059151 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=292 at=1788326059171 throughput_bps=741934 planner_Bps=92741 slot_demand=true action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=293 at=1788326059191 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=294 at=1788326059208 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=295 at=1788326059230 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=296 at=1788326059247 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=297 at=1788326059275 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=298 at=1788326059325 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=299 at=1788326059344 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=300 at=1788326059431 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=301 at=1788326059530 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=302 at=1788326059630 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=303 at=1788326059730 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=304 at=1788326059750 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=305 at=1788326059830 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=306 at=1788326059931 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=307 at=1788326060080 throughput_bps=741934 planner_Bps=92741 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_LONG_CANCEL sequence=126 selected=45d269df563ddd17c40a22b4f4fcc144 executed=45d269df563ddd17c40a22b4f4fcc144 bytes=0
WARP_LONG_CANCEL peerClosed=true originAcceptedBytes=0
WARP_HLS_STATE stage=beforeSwipe active=0 projected=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1 history=36:preparing:0:null|38:preparing:0:null|41:preparing:109:null|131:preparing:109:null|136:preparing:480:null|225:preparing:480:null|229:preparing:1290:null|322:preparing:1290:null|328:startable:6347:baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1
WARP_HLS_RESERVE revision=230 mode=Safety target=2 ordered=1 ready=1 kinds=WarpReserveCandidateKind.hls|WarpReserveCandidateKind.progressive candidates=d12abe0814caafd20be1206aedf87d81|a38f559f6a7c169daa152f61ba2a383d
WARP_HLS_FRAME delivery=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4 representation=b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b session=ad046228737de02fb581bcfa571a766bcead51ebddc14bb13ec5cb59b7c2e18c generation=2 frameUs=68855 gatewayAcquisitions=1 activeLeases=1 rootRequests=1 selectedRequests=1 alternateRequests=0 initRequests=1 segment0Requests=1 rescued=false
WARP_HLS_STATE stage=afterThird active=2 projected=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1 history=36:preparing:0:null|38:preparing:0:null|41:preparing:109:null|131:preparing:109:null|136:preparing:480:null|225:preparing:480:null|229:preparing:1290:null|322:preparing:1290:null|328:startable:6347:baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1
WARP_HLS_AUTH structural=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1 request=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1 lease=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1 gatewayAcquisitions=1
WARP_HLS_CLEANUP delivery=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4 authority=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4:b723b0f10e9a68683e88afbe8cbf9eb32991aadd407bb0537401313ea947ca4b:1 lifecycle=initializing|initialized|firstFrameRendered|released activeLeases=0
```

### First-frame lines

```
WARP_HLS_CLEANUP delivery=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922cad98d4 authority=baffc99500bd61b48125c8394a1e8df16614dda272b3f096440d12922ca
```

## Run `20260902T051836Z-314742d3db-22e0d933-origin-timeout-peer-close-2`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_origin_timeout_fallback_video_test.dart -d 22e0d933 `
- commit: `314742d3db66eb081ef72e8cc46d7e17ba9a225d` (dirty_files=10)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +0 -1 | FAIL |

### WARP markers

```
WARP_DECISION_HISTORY retained=64 first=410 latest=473
WARP_DECISION sequence=410 at=1788326367663 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=411 at=1788326367763 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=412 at=1788326367863 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=413 at=1788326367963 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=414 at=1788326368040 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=415 at=1788326368062 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=416 at=1788326368162 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=417 at=1788326368262 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=418 at=1788326368363 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=419 at=1788326368463 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=420 at=1788326368540 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=421 at=1788326368563 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=422 at=1788326368622 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=423 at=1788326368662 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=424 at=1788326368763 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=425 at=1788326368864 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=426 at=1788326368878 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=427 at=1788326368963 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=428 at=1788326369040 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=429 at=1788326369072 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=430 at=1788326369163 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=431 at=1788326369263 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=432 at=1788326369363 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=433 at=1788326369463 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=434 at=1788326369540 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=435 at=1788326369562 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=436 at=1788326369619 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=437 at=1788326369663 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=438 at=1788326369764 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=439 at=1788326369863 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=440 at=1788326369963 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=441 at=1788326370040 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=442 at=1788326370062 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=443 at=1788326370084 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=444 at=1788326370141 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=445 at=1788326370163 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=446 at=1788326370263 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=447 at=1788326370363 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=448 at=1788326370463 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=449 at=1788326370540 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=450 at=1788326370562 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=451 at=1788326370624 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=452 at=1788326370663 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=453 at=1788326370763 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=454 at=1788326370863 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=455 at=1788326370963 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=456 at=1788326370978 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=457 at=1788326371040 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=458 at=1788326371062 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=459 at=1788326371163 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=460 at=1788326371263 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=461 at=1788326371363 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=462 at=1788326371463 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=463 at=1788326371540 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=464 at=1788326371565 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=465 at=1788326371626 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=466 at=1788326371663 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=467 at=1788326371763 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=468 at=1788326371863 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=469 at=1788326371963 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=470 at=1788326372040 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=471 at=1788326372062 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=472 at=1788326372163 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=473 at=1788326372274 throughput_bps=2360251 planner_Bps=295031 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
```


## Run `20260902T051938Z-314742d3db-22e0d933-origin-timeout-peer-close-3`

- command: `flutter test --no-uninstall --no-pub integration_test/warp_feed_origin_timeout_fallback_video_test.dart -d 22e0d933 `
- commit: `314742d3db66eb081ef72e8cc46d7e17ba9a225d` (dirty_files=11)
- device: ro.product.model=M2012K11AG ro.build.version.release=13 ro.kernel.qemu= 
- exit: 1

| test file | last progress | result |
|---|---|---|
| integration_test/warp_feed_origin_timeout_fallback_video_test.dart | +0 -1 | FAIL |

### WARP markers

```
WARP_DECISION_HISTORY retained=64 first=404 latest=467
WARP_DECISION sequence=404 at=1788326430918 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=405 at=1788326431018 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=406 at=1788326431032 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=407 at=1788326431118 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=408 at=1788326431195 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=409 at=1788326431217 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=410 at=1788326431318 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=411 at=1788326431417 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=412 at=1788326431517 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=413 at=1788326431532 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=414 at=1788326431617 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=415 at=1788326431718 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=416 at=1788326431818 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=417 at=1788326431918 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=418 at=1788326432017 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=419 at=1788326432033 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=420 at=1788326432118 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=421 at=1788326432194 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=422 at=1788326432217 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=423 at=1788326432233 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=424 at=1788326432321 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=425 at=1788326432419 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=426 at=1788326432519 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=427 at=1788326432533 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=428 at=1788326432618 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=429 at=1788326432718 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=430 at=1788326432818 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=431 at=1788326432918 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=432 at=1788326433018 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=433 at=1788326433032 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=434 at=1788326433118 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=435 at=1788326433194 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=436 at=1788326433218 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=437 at=1788326433317 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=438 at=1788326433418 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=439 at=1788326433518 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=440 at=1788326433532 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=441 at=1788326433619 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=442 at=1788326433636 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=443 at=1788326433677 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=444 at=1788326433718 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=445 at=1788326433818 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=446 at=1788326433917 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=447 at=1788326434017 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=448 at=1788326434033 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=449 at=1788326434117 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=450 at=1788326434196 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=451 at=1788326434219 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=452 at=1788326434317 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=453 at=1788326434334 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=454 at=1788326434417 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=455 at=1788326434518 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=456 at=1788326434532 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=457 at=1788326434617 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=458 at=1788326434717 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=459 at=1788326434817 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=460 at=1788326434918 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=461 at=1788326435017 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=462 at=1788326435033 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=463 at=1788326435117 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=464 at=1788326435196 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=465 at=1788326435236 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=466 at=1788326435335 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
WARP_DECISION sequence=467 at=1788326435435 throughput_bps=3083335 planner_Bps=385416 slot_demand=false action=null outcome=succeeded detail=null bytes=0 elapsed_ms=0 selected=null:null:null:null:null-null:target=null executed=null:null:null-null
```

