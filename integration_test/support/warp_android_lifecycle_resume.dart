part of 'warp_android_lifecycle_scenario.dart';

extension _WarpAndroidResume on WarpAndroidLifecycleScenario {
  Future<void> _resume(WidgetTester tester, _LifecyclePlayback initial) async {
    await waitForAndroidLifecycleEvidence(
      tester,
      () => lifecycle.hasResumedAfterBackground,
    );
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
}
