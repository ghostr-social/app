part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyLinkWait on WarpFeedPlaybackJourney {
  Future<ProgressiveOriginLinkProfile> waitForBandwidthTrigger(
    WidgetTester tester,
    ProgressiveOriginBandwidthTrigger trigger,
  ) async {
    await _wait(tester, () => trigger.isReached);
    if (!trigger.timedOut) return trigger.profile!;
    await reportSchedulingEvidence();
    fail('The shared-link bandwidth trigger timed out.');
  }

  Future<void> waitForChunkGate(
    WidgetTester tester,
    ProgressiveOriginChunkGate gate,
  ) async {
    await _wait(tester, () => gate.isReached);
    if (!gate.timedOut) return;
    await reportSchedulingEvidence();
    fail('The shared-link chunk gate timed out.');
  }

  Future<ProgressiveOriginLinkWindow> waitForConfirmedLinkWindow(
    WidgetTester tester,
    int generation, {
    required Duration minimumDuration,
  }) async {
    await _wait(
      tester,
      () => resources.origin.linkWindow(generation).duration >= minimumDuration,
    );
    return resources.origin.linkWindow(generation);
  }

  Future<Set<int>> waitForActiveOriginRequests(WidgetTester tester) async {
    await _wait(
      tester,
      () => resources.origin.activeIncompleteRequestSequences.isNotEmpty,
    );
    return resources.origin.activeIncompleteRequestSequences;
  }

  Future<void> waitForRequestProfiles(
    WidgetTester tester,
    int request,
    Set<int> generations,
  ) {
    return _wait(
      tester,
      () => resources.origin.requestSpansProfiles(request, generations),
    );
  }
}
