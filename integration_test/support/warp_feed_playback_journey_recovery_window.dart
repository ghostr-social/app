part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyRecoveryWindow on WarpFeedPlaybackJourney {
  Future<WarpReadyWindow> waitForRecoveryFrontierWindow(
    WidgetTester tester,
    BigInt generation, {
    required PlaybackDeliveryId currentDeliveryId,
    int minimumDepth = 2,
    int afterSequence = 0,
  }) {
    return _waitForWindow(tester, (
      generation: generation,
      minimumDepth: minimumDepth,
      afterRevision: 0,
      afterSequence: afterSequence,
      currentDeliveryId: currentDeliveryId,
      goal: WarpReadyWindowGoal.recoveryFrontier,
    ));
  }
}
