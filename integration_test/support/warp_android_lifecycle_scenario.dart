import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_android_lifecycle_probe.dart';
import 'warp_android_lifecycle_wait.dart';
import 'warp_feed_playback_journey.dart';
import 'warp_feed_player_stage_probe.dart';

part 'warp_android_lifecycle_quiescence.dart';

final class WarpAndroidLifecycleScenario {
  const WarpAndroidLifecycleScenario._(this.lifecycle, this.journey);

  static Future<WarpAndroidLifecycleScenario> start() async {
    final journey = await WarpFeedPlaybackJourney.start();
    return WarpAndroidLifecycleScenario._(
      WarpAndroidLifecycleProbe.attach(),
      journey,
    );
  }

  final WarpAndroidLifecycleProbe lifecycle;
  final WarpFeedPlaybackJourney journey;

  Future<void> run(WidgetTester tester) async {
    final initial = await _openInitial(tester);
    final generation = initial.session.generation;
    debugPrint('WARP_ANDROID_LIFECYCLE_READY session=$generation');
    await _background(tester, initial);
    await _resume(tester, initial);
    await _teardown(tester);
  }

  Future<_LifecyclePlayback> _openInitial(WidgetTester tester) async {
    await tester.pumpWidget(journey.app);
    journey.load();
    await journey.waitForCaption(tester, 0);
    final focus = await journey.waitForPublishedFocus(tester, 0);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
    return _evidence(focus);
  }

  _LifecyclePlayback _evidence(PlaybackFocus focus) {
    final session = journey.telemetry.probe.sessionFor(focus)!;
    final presentation = journey.telemetry.probe.presentationFor(focus)!;
    final stage = journey.playerStages.forPresentation(
      session.deliveryId,
      presentation.elapsed,
    )!;
    return (focus: focus, session: session, stage: stage);
  }

  Future<void> _background(
    WidgetTester tester,
    _LifecyclePlayback initial,
  ) async {
    await lifecycle.backgrounded.timeout(const Duration(seconds: 30));
    await waitForAndroidLifecycleEvidence(
      tester,
      () =>
          journey.telemetry.probe.deactivations.contains(initial.session) &&
          initial.stage.releasedAt != null,
    );
    final states = lifecycle.evidence;
    debugPrint('WARP_ANDROID_LIFECYCLE_BACKGROUND states=$states');
  }

  Future<void> _resume(WidgetTester tester, _LifecyclePlayback initial) async {
    await lifecycle.requireResumedAfterBackground(const Duration(seconds: 30));
    debugPrint('WARP_ANDROID_LIFECYCLE_RESUMED states=${lifecycle.evidence}');
    await journey.waitForCaption(tester, 1);
    final focus = await journey.waitForPublishedFocus(
      tester,
      1,
      afterSequence: initial.focus.sequence,
    );
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
    final resumed = _evidence(focus);
    expect(resumed.session, isNot(initial.session));
    expect(resumed.stage.firstFrameAt, isNotNull);
    expect(find.text('Video unavailable'), findsNothing);
    debugPrint(
      'WARP_ANDROID_LIFECYCLE_EVIDENCE states=${lifecycle.evidence} '
      'initial=${initial.session.generation} '
      'resumed=${resumed.session.generation}',
    );
  }

  Future<void> close() async {
    lifecycle.close();
    await journey.close();
  }
}
