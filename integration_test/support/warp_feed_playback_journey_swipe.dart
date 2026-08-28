part of 'warp_feed_playback_journey.dart';

typedef WarpReadyWindow = ({
  WarpPlanEvidence plan,
  WarpFeedPreparationObservation snapshot,
});

extension WarpFeedPlaybackJourneySwipe on WarpFeedPlaybackJourney {
  Future<WarpReadyWindow> waitForReadyBurstWindow(
    WidgetTester tester,
    BigInt generation, {
    required PlaybackDeliveryId currentDeliveryId,
    int minimumDepth = 1,
    int afterSequence = 0,
  }) async {
    return _waitForWindow(tester, (
      generation: generation,
      minimumDepth: minimumDepth,
      afterRevision: 0,
      afterSequence: afterSequence,
      currentDeliveryId: currentDeliveryId,
      goal: WarpReadyWindowGoal.consumeBurst,
    ));
  }

  bool isReadyIn(
    WarpFeedPreparationObservation snapshot,
    PlaybackFocus focused,
  ) {
    final session = telemetry.probe.sessionFor(focused);
    if (session == null) return false;
    final stage = playerStages.preparedFor(
      session.deliveryId,
      focused.startedAt,
    );
    return stage != null &&
        snapshot.has(stage.authority, PlaybackPreparationReadiness.ready);
  }

  Future<WarpReadyWindow> waitForReplenishment(
    WidgetTester tester,
    PlaybackFocus focused, {
    required int afterRevision,
  }) async {
    final generation = focus.generationFor(focused)!;
    final session = telemetry.probe.sessionFor(focused)!;
    return _waitForWindow(tester, (
      generation: generation,
      minimumDepth: 1,
      afterRevision: afterRevision,
      afterSequence: 0,
      currentDeliveryId: session.deliveryId,
      goal: WarpReadyWindowGoal.fillTarget,
    ));
  }
}
