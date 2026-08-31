part of 'warp_feed_playback_journey.dart';

typedef _WarpPairWaitQuery = ({
  bool Function(WarpDecisionRecord, WarpPlanEvidence) accepts,
  int afterSequence,
  int afterRevision,
  Duration timeout,
});

extension WarpFeedPlaybackJourneyPairWait on WarpFeedPlaybackJourney {
  Future<WarpDecisionPlanPair> waitForDecisionPlanPair(
    WidgetTester tester,
    bool Function(WarpDecisionRecord, WarpPlanEvidence) accepts, {
    int afterSequence = 0,
    int afterRevision = 0,
    Duration timeout = const Duration(seconds: 15),
  }) {
    return _WarpPairWait(this, tester, (
      accepts: accepts,
      afterSequence: afterSequence,
      afterRevision: afterRevision,
      timeout: timeout,
    )).run();
  }
}

final class _WarpPairWait {
  _WarpPairWait(this.journey, this.tester, this.query);
  final WarpFeedPlaybackJourney journey;
  final WidgetTester tester;
  final _WarpPairWaitQuery query;
  final List<WarpPlanEvidence> _plans = [];
  int? _cursor;

  Future<WarpDecisionPlanPair> run() async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < query.timeout) {
      final page = await _drainPlanBacklog();
      final pair = await _matchingPair();
      if (pair != null) return pair;
      if (!page.hasMore) await journey._tickAndSample(tester);
    }
    await journey.reportSchedulingEvidence();
    await _reportPairDiagnostics();
    fail('WARP decision/plan pair timed out after ${query.timeout}.');
  }

  Future<WarpPlanPage> _drainPlanBacklog() async {
    var page = await _samplePlans();
    final target = page.latestRetainedRevision;
    while ((_cursor ?? query.afterRevision) < target) {
      page = await _samplePlans();
    }
    return page;
  }

  Future<WarpPlanPage> _samplePlans() async {
    final cursor = _cursor ?? query.afterRevision;
    final evidence = await journey.evidence.page(afterRevision: cursor);
    final page = evidence.planPage;
    if (page.cursorTruncated) {
      fail('WARP pair evidence cursor was truncated after $cursor.');
    }
    _plans.addAll(page.records);
    if (page.records.isNotEmpty) _cursor = page.records.last.revision;
    return page;
  }

  Future<WarpDecisionPlanPair?> _matchingPair() async {
    final decisions = await journey.evidence.decisions();
    return warpFirstDecisionPlanPair((
      decisions: decisions.records,
      plans: _plans,
      afterSequence: query.afterSequence,
      afterRevision: query.afterRevision,
      accepts: query.accepts,
    ));
  }
}
