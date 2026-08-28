part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyDecisionWait on WarpFeedPlaybackJourney {
  Future<WarpDecisionRecord> waitForDecision(
    WidgetTester tester,
    bool Function(WarpDecisionRecord decision) predicate, {
    int afterSequence = 0,
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < timeout) {
      final decisions = await evidence.decisions();
      for (final decision in decisions.records) {
        if (decision.sequence > afterSequence && predicate(decision)) {
          return decision;
        }
      }
      await _tickAndSample(tester);
    }
    await reportSchedulingEvidence();
    fail('WARP decision evidence timed out after $timeout.');
  }
}
