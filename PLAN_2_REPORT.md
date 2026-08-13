# PLAN 2 player contract report

Date: 2026-08-13

## Decision

The locked `video_player` stack exposes the required native facts and controls.
The failing contracts were all Ghostr Dart adapter or deterministic-runner
issues. No Android or iOS player implementation, plugin fork, package override,
decoder, network client, cache, scheduler, or player pool was added.

Reviewed locked packages:

- `video_player` 2.13.0
- `video_player_android` 2.12.0
- `video_player_avfoundation` 2.11.0
- `video_player_platform_interface` 6.9.0

## Capability map

| Required contract | Dart source | Android Media3 source | iOS AVPlayer source | Result |
| --- | --- | --- | --- | --- |
| Initialize and become ready | `VideoPlayerController.initialize` and `VideoEventType.initialized` | `ExoPlayerEventListener.onPlaybackStateChanged(STATE_READY)` calls `maybeSendInitialized` | `FVPVideoPlayer` observes `AVPlayerItem.status` and calls `reportInitialized` | Existing plugin API |
| Play, pause, seek, and playback rate | Controller `play`, `pause`, `seekTo`, and `setPlaybackSpeed`; `VideoPlayerValue.playbackSpeed` | `VideoPlayer` delegates to `ExoPlayer.play`, `pause`, `seekTo`, and playback parameters | `FVPVideoPlayer` delegates to `AVPlayer.play`, `pause`, `seekToTime`, and `rate` | Existing plugin API |
| Completion and error | `VideoPlayerValue.isCompleted`, `hasError`, and `errorDescription` | `STATE_ENDED` becomes `completed`; `onPlayerError` emits `VideoError` | `AVPlayerItemDidPlayToEndTimeNotification` emits completion; failed item status emits an error | Existing plugin API |
| Position | `VideoPlayerValue.position` | `VideoPlayer.getCurrentPosition` reads `ExoPlayer.getCurrentPosition` | `FVPVideoPlayer.position` reads `AVPlayer.currentTime` | Existing plugin API |
| Buffered ranges and contiguous buffered-ahead | `VideoPlayerValue.buffered`; Ghostr walks only ranges contiguous with the current position | `getBufferedPosition` is polled and mapped to `[0, bufferedPosition]` | `loadedTimeRanges` KVO emits every `CMTimeRange` | Existing plugin fact; Dart derivation tested |
| Individually observable stall start/end | `VideoEventType.bufferingStart` and `bufferingEnd` update `VideoPlayerValue.isBuffering` | distinct `PlaybackStateChangeEvent` values map `STATE_BUFFERING` and the following non-buffering state to start/end | `playbackLikelyToKeepUp` KVO emits start and end separately | Existing plugin fact; Dart queue corrected |
| Current/next preparation with one audible player | independent controllers plus `setVolume`, `play`, and `pause` | one Media3 `ExoPlayer` per player ID | one `AVPlayer` per player ID | Existing controls; serialized Dart handoff added |
| Disposal | `VideoPlayerController.dispose` cancels events and invokes platform disposal | listener/timer cleanup followed by `ExoPlayer.release` and surface release | notifications/KVO are removed and the current item is replaced with `nil` | Existing plugin control |
| Delivery identity and generation | Ghostr's typed `PlaybackSession(videoId, deliveryId, generation)` owns the domain binding | native player ID keeps events scoped to one Media3 instance | native player ID keeps events scoped to one AVPlayer instance | App-domain responsibility; typed Dart binding added |
| Covered/destroyed cleanup and stale-event rejection | Ghostr ends observation, detaches the listener, retires the session, and awaits controller disposal | disposed event subscription and `_isDisposed` checks prevent later polling events | `_disposed` plus observer removal prevents later callbacks | Existing native cleanup; Dart lifecycle corrected |

## Failing contracts and corrections

| Red contract | Failure observed | Smallest correction |
| --- | --- | --- |
| Delivery-bound session | No typed loopback delivery identity existed at the player boundary | Added `PlaybackDeliveryId`, derived only from typed proxied HLS/progressive sources, and made it part of `PlaybackSession` |
| Generation identity across adapter reconstruction | A replacement adapter reused generation `1` | Moved monotonic generation ownership into the long-lived telemetry port; every active native replacement opens a new session |
| Prepared predecessor generation order | Preparation reserved generation `1`, current playback activated generation `2`, then reverse traversal tried to activate stale generation `1` | Defer session allocation until the prepared controller actually activates, so activation order and generation order are identical |
| Stall boundary ordering | A queued `starting, playing, stalled, playing` sequence arrived as only `starting, playing` | Queue observations per generation and coalesce only consecutive samples with the same phase |
| Current/next audible handoff | A delayed pause allowed two initialized controllers to be audible together | Added one serialized mute, play, and conditional-unmute handoff shared by surfaces from the same playback port |
| Covered-player cleanup | Covering active playback produced zero platform disposals | On cover, invalidate intent, emit inactive, detach, clear the session/controller, and dispose the platform player |
| Deterministic contract-build storage | The locked Rust plugin's default dev debug artifacts exhausted the host volume during repeated device builds | Added a runner contract first, then set `CARGO_PROFILE_DEV_DEBUG=0` in both device runners; this affects contract builds only, not product or release behavior |

Initialization, play, pause, seek, completion, error, position, playback rate,
buffered ranges, and default disposal passed against the locked plugin without a
native extension. Stale events are also rejected after a generation is retired.
A focused regression confirms that the existing per-send exception isolation
continues draining telemetry after a reporter failure.

## Tests added or changed

- Device contracts: lifecycle/metrics, initialization error, stall boundaries,
  single-audible preparation, reverse prepared-generation order, and
  delivery/generation identity in the six
  `integration_test/video_player_*_contract_test.dart` files.
- Host adapter regressions: delivery identity, generation replacement, stale
  events, covered cleanup, current/next handoff, phase coalescing, session
  ordering, reporter failure, lifecycle/phases, and prepared command failures
  under `test/core/media`, `test/media`, and `test/video_inventory`.
- Deterministic runner contracts:
  `test/tool/video_player_contract_target_test.sh` and
  `test/tool/android_agent_avd_architecture_test.sh`.
- The shared device fixture/probe and existing impairment tests now use the
  typed delivery/session model; their impairment behavior was not changed.

## Deterministic device commands

- `make video-player-contract-android` selects the repository-owned
  `Ghostr_Agent_API_37.1` AVD on `emulator-5580`, chooses `arm64-v8a` on this
  host (`x86_64` on Intel), enforces 16 GB storage, boots it, and tears it down
  only when the runner started it.
- `make video-player-contract-ios` finds or creates
  `Ghostr_Player_Contract` on the newest available iOS runtime, boots it, runs
  the contract by UDID, and shuts it down only when the runner started it.
- `make video-player-contract` runs the target contract test and both device
  suites. It needs no caller-supplied serial, UDID, signing choice, or manual
  interaction.

Both device runners bound Cargo dev debug information for these transient
contract builds. This keeps the locked Rust plugin build substantially smaller
without changing application behavior or the release profile.

The locked `flutter_secure_storage` and `rust_lib_ghostr` plugins do not support
Swift Package Manager, so the iOS runner uses the repository CocoaPods setup.
Flutter's current iOS project migration (iOS 15 and scene lifecycle) is checked
in so the command remains reproducible.

Host setup note: the incompatible x86_64 repository AVD was preserved at
`/Users/gp/.android/avd/incompatible-Ghostr_Agent_API_37.1-20260813132456-64627`;
the runner created the compatible ARM AVD without `--force` or wiping data.
Workspace-specific generated Android/iOS build output and Xcode `Runner`
DerivedData were removed during disk recovery; all are reproducible caches and
will regenerate on the next relevant build. The repository AVD and its data
were preserved.

## Verification

| Command | Result |
| --- | --- |
| `flutter test --no-pub` | Passed: 1,272 tests |
| `make test-coverage` | Passed: 1,272 tests |
| `make coverage-summary` | Passed: 99.65% (10,245/10,281), all executable Dart sources represented, all 383 per-file gates passed |
| `make video-player-contract-android` | Passed: 6/6 Media3 contracts through the canonical aggregate runner |
| `make video-player-contract-ios` | Passed: 6/6 AVPlayer contracts through the canonical iOS runner |
| `make video-player-contract` | Target contract and Android 6/6 passed; the sequential iOS stage was prevented from building by host `ENOSPC` before an assertion ran. The same iOS child command passed 6/6 independently |
| `sh test/tool/video_player_contract_target_test.sh` | Passed, including the debug-artifact bound on both runners |
| `sh test/tool/android_agent_avd_architecture_test.sh` | Passed |
| `git diff --check` | Passed |
| `flutter analyze --no-pub` | Unchanged baseline exit 1: only three pre-existing `unawaited_return_in_try_block` warnings outside this plan's boundaries |

The unchanged analyzer warnings are in:

- `lib/app/production_video_delivery_infrastructure.dart:31`
- `lib/features/video_catalog/data/rust_feed_remote_watcher.dart:29`
- `lib/features/video_catalog/domain/hybrid_video_reader.dart:41`

The latter two are feed code explicitly excluded by this plan. No Rust
downloader, gateway, adaptive-policy, or feed file was changed.
