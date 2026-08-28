part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyNetwork on WarpFeedPlaybackJourney {
  Future<DeliveryNetworkStatus> publishNetwork(
    DeliveryNetworkClass networkClass,
  ) async {
    final status = graph.network.publish(networkClass);
    await graph.delivery.networkStatusRuntime!.settled;
    return status;
  }

  Future<WarpPlanEvidence> waitForPlan(
    WidgetTester tester,
    bool Function(WarpPlanEvidence plan) predicate, {
    int afterRevision = 0,
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final watch = Stopwatch()..start();
    var cursor = afterRevision;
    while (watch.elapsed < timeout) {
      final page = await evidence.page(afterRevision: cursor);
      if (page.planPage.cursorTruncated) {
        fail('WARP plan evidence cursor was truncated after $cursor.');
      }
      for (final plan in page.planPage.records) {
        if (predicate(plan)) return plan;
      }
      if (page.planPage.records.isNotEmpty) {
        cursor = page.planPage.records.last.revision;
      }
      if (!page.planPage.hasMore) await _tickAndSample(tester);
    }
    await reportSchedulingEvidence();
    fail('WARP plan evidence timed out after $timeout.');
  }
}
