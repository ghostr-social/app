# PLAN 2 — Prove what `video_player` lacks; extend only that

## Goal

Determine whether the existing Flutter player layer exposes every playback
fact and lifecycle control listed below. Do not replace a working native
playback stack.

`video_player` already delegates to Media3 on Android and AVPlayer on iOS. Rust
continues to download and serve loopback URLs; the native players decode and
render them. This branch concerns observation and lifecycle control only.

## Automatic decision rule

First add contract tests using the existing integration harness and the current
locked plugin versions. The required contract is:

- initialization, play, pause, seek, completion, error, and disposal;
- position, playback rate, and contiguous buffered-ahead duration derived from
  buffered ranges;
- individually observable stall start/end transitions, not a coalesced guess;
- current/next preparation with an automated assertion that at most one
  controller is both unmuted and playing;
- every observation tied to the correct loopback delivery ID and player
  generation;
- deterministic cleanup—platform player released and no later event attributed
  to the old generation—when a feed surface is covered or destroyed.

If the plugin exposes a required fact but Ghostr's Dart adapter maps it
incorrectly, fix only that adapter after its failing test. If the complete
contract passes, make no Android/iOS implementation.

If a required contract fails because the plugin API cannot expose the native
fact or control, implement only the missing typed bridge through the smallest
extension seam supported by the locked packages. Reuse their existing Media3
and AVPlayer instances. A package fork or local override is a last resort, not
the default design. Do not add another decoder, network client, cache,
scheduler, or player pool.

The test result—not a human decision—selects the path.

## Work

1. Read the locked `video_player`, `video_player_android`, and
   `video_player_avfoundation` versions and their platform implementations.
2. Write a capability table mapping every required fact/control to its Dart,
   Android Media3, and iOS AVPlayer source. Mark a gap only after a failing
   automated contract demonstrates it.
3. Add generation-safe Dart adapter tests before any adapter correction.
4. Run the contract on Android AVD and iOS simulator with deterministic local
   fixtures. Add stable commands that select/create the virtual target
   automatically; no supplied serial, UDID, signing, or manual step.
5. For each proven gap, add the smallest typed platform event or command, with
   a platform test first and a Dart adapter test second.
6. Rerun the same contract against the result and record whether any native
   extension was necessary.

## Boundaries

- Do not edit the Rust downloader, gateway, adaptive policy, or feed.
- Do not fork all of `video_player` merely to expose one callback.
- Do not invent bitrate, stall reason, buffer, or lifecycle evidence that the
  platform did not emit.
- A controller keeps the same platform player instance for its lifetime; never
  swap a live player.

## Done

The branch ends when the committed contract commands pass on Android AVD and
iOS simulator. Write one concise report: what the existing plugin already
provided, which contracts failed, the exact Dart correction or native bridge
added for each failure, and command results. A redundant native player
implementation is a failure of this plan.
