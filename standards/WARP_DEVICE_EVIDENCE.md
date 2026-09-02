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

## Status summary (2026-09-01)

| run | head | result |
|---|---|---|
| `make video-delivery-target-contract-test` | `1c03551b` | pass |
| `make video-android-physical-evidence` (26-file matrix + offline-restart + lifecycle) | `1c03551b` | **stopped after 12 of 26 files**: 11 pass, 1 fail (`warp_feed_adaptive_warm_back_video_test`), 14 files plus offline-restart and lifecycle not run |
| targeted validation of the ETA and origin-exhaustion slices (4 files) | `6d015337` | see the run section below once it lands |

Why the matrix was stopped: on this Mac every integration test file makes Gradle re-run the
Rust cross-compile, at 26 minutes per file (the `build/` directory was a symlink onto an external
SSD), so the full target needs roughly nine hours. The 12 files that ran cover bandwidth drop,
packet loss, high RTT, rapid swipes, held responses, manifest retry, progressive delivery, loop
reopen, feed playback, visible motion and rapid-swipe instrumentation. The remaining files must be
run on the next session; `build/` is now a plain directory on the internal disk to make that
feasible.

Diagnosis of the one failure (`warp_feed_adaptive_warm_back_video_test`, "physical feed reuses
all three recent decoded players when capacity is free"): the scenario timed out after 15 s with
`playbackErrors=0`, `readyDepth=3`, and its focus trace shows
`transportRescue:etaUnavailable` at 175 ms. On head `1c03551b` progressive delivery snapshots never
carry an ETA (`eta_ms: None`), so the §9.8 grace decision could not wait and rescued off the
intended item, which the scenario forbids. Commit `9e84aa28` (causal progressive ETA) is the fix
under validation in the targeted run.

The `warp_feed_startup_singleflight_video_test` row in the matrix table is not a test failure: the
driver was stopped while that file was loading, and flutter reports the interrupted load as a
failure ("Failed to load ..."). It has not been run.

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

