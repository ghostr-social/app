part of 'warp_feed_playback_journey.dart';

typedef _WarpWindowQuery = ({
  BigInt generation,
  int minimumDepth,
  int afterRevision,
  int afterSequence,
  PlaybackDeliveryId? currentDeliveryId,
  WarpReadyWindowGoal goal,
});

extension WarpFeedPlaybackJourneyWindow on WarpFeedPlaybackJourney {
  Future<WarpReadyWindow> _waitForWindow(
    WidgetTester tester,
    _WarpWindowQuery query,
  ) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < const Duration(seconds: 15)) {
      final match = await _matchingWindow(query);
      if (match != null) return match;
      await _tickAndSample(tester);
    }
    debugPrint(
      'WARP_WINDOW goal=${query.goal.name} generation=${query.generation} '
      'depth=${query.minimumDepth} after_revision=${query.afterRevision} '
      'after_sequence=${query.afterSequence} '
      'current=${query.currentDeliveryId?.value}',
    );
    await reportSchedulingEvidence();
    fail(_timeoutEvidence(const Duration(seconds: 15)));
  }

  Future<WarpReadyWindow?> _matchingWindow(_WarpWindowQuery query) async {
    final snapshots = preparation.observations;
    final index = warpNewestCausalEvidenceIndex(
      history: snapshots
          .map(
            (item) =>
                (revision: item.revision.toInt(), sequence: item.sequence),
          )
          .toList(growable: false),
      afterRevision: query.afterRevision,
      afterSequence: query.afterSequence,
    );
    if (index == null) return null;
    final snapshot = snapshots[index];
    if (!_matchesCurrent(snapshot, query.currentDeliveryId)) return null;
    final revision = snapshot.revision.toInt();
    final plan = await _planAt(revision, query.generation);
    if (plan == null || !_acceptsWindow(snapshot, plan, query)) return null;
    return (plan: plan, snapshot: snapshot);
  }

  bool _matchesCurrent(
    WarpFeedPreparationObservation snapshot,
    PlaybackDeliveryId? currentDeliveryId,
  ) {
    return currentDeliveryId == null ||
        snapshot.currentDeliveryId == currentDeliveryId;
  }

  bool _acceptsWindow(
    WarpFeedPreparationObservation snapshot,
    WarpPlanEvidence plan,
    _WarpWindowQuery query,
  ) {
    return warpReadyEvidenceAccepted(
      revision: (preparation: snapshot.revision.toInt(), plan: plan.revision),
      sequence: (observation: snapshot.sequence, after: query.afterSequence),
      readiness: (
        contiguous: snapshot.contiguousReadyDepth,
        ordered: plan.plan.readyReserve.orderedReady,
        minimum: query.minimumDepth,
        target: plan.plan.readyReserve.target,
        candidateCount: plan.plan.readyReserve.candidateCount,
        goal: query.goal,
      ),
    );
  }

  Future<WarpPlanEvidence?> _planAt(int revision, BigInt generation) async {
    final page = await evidence.page(afterRevision: revision - 1, limit: 1);
    if (page.planPage.records.isEmpty) return null;
    final plan = page.planPage.records.single;
    return plan.coversFocusGeneration(generation) ? plan : null;
  }
}
