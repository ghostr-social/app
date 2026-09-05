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
part 'warp_android_lifecycle_resume.dart';

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

  Future<void> close() async {
    lifecycle.close();
    await journey.close();
  }
}
