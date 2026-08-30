part of 'warp_feed_playback_journey.dart';

typedef _OriginCoverageDelta = ({
  WarpOriginUse before,
  WarpOriginUse after,
  ProgressiveOriginCoverage prior,
  ProgressiveOriginCoverage finalCoverage,
  String evidence,
});

extension WarpFeedPlaybackJourneyOriginAssertions on WarpFeedPlaybackJourney {
  void verifyReplayOriginUse(
    WarpOriginSnapshot before,
    WarpOriginSnapshot after,
  ) {
    final coverages = <ProgressiveOriginCoverage>[];
    for (final entry in before.entries) {
      coverages.add(
        _expectReplayOriginUse(entry.key, entry.value, after[entry.key]!),
      );
    }
    _expectAggregateCancellationOverlap(coverages);
  }

  ProgressiveOriginCoverage _expectReplayOriginUse(
    String id,
    WarpOriginUse before,
    WarpOriginUse after,
  ) {
    final prior = resources.origin.coverageFor(
      id,
      requestCount: before.requests,
    );
    final finalCoverage = resources.origin.coverageFor(id);
    final added = resources.origin
        .requestsFor(id)
        .skip(before.requests)
        .toList();
    final evidence =
        '$id prior=${_coverageEvidence(prior)} '
        'final=${_coverageEvidence(finalCoverage)}; '
        '${originRequestEvidence([id])}';
    _expectCoverageIntegrity(prior, finalCoverage, evidence);
    _expectCoverageDelta((
      before: before,
      after: after,
      prior: prior,
      finalCoverage: finalCoverage,
      evidence: evidence,
    ));
    _expectReplayRequests(added, prior, evidence);
    return finalCoverage;
  }

  void _expectAggregateCancellationOverlap(
    List<ProgressiveOriginCoverage> coverages,
  ) {
    expect(
      progressiveReplayCancellationOverlapWithin(
        coverages,
        budgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
      reason:
          'Replay cancellation overlap exceeded the scenario budget: '
          '${coverages.map((item) => item.cancellationAttributedDuplicateBytes)}',
    );
  }

  void _expectCoverageIntegrity(
    ProgressiveOriginCoverage prior,
    ProgressiveOriginCoverage finalCoverage,
    String evidence,
  ) {
    expect(
      prior.hasReplayIntegrityWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
      reason: evidence,
    );
    expect(
      finalCoverage.hasReplayIntegrityWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
      reason: evidence,
    );
  }

  String _coverageEvidence(ProgressiveOriginCoverage coverage) {
    return 'missing=${coverage.missingRanges},'
        'network=${coverage.networkBytes},unique=${coverage.uniqueBytes},'
        'duplicate=${coverage.duplicateBytes},'
        'completedDuplicate=${coverage.completedDuplicateBytes},'
        'canceledDuplicate=${coverage.cancellationAttributedDuplicateBytes}';
  }

  void _expectCoverageDelta(_OriginCoverageDelta delta) {
    expect(
      delta.after.requests,
      greaterThanOrEqualTo(delta.before.requests),
      reason: delta.evidence,
    );
    expect(
      delta.before.bytes,
      delta.prior.networkBytes,
      reason: delta.evidence,
    );
    expect(
      delta.after.bytes,
      delta.finalCoverage.networkBytes,
      reason: delta.evidence,
    );
    expect(
      delta.after.bytes - delta.before.bytes,
      delta.finalCoverage.uniqueBytes -
          delta.prior.uniqueBytes +
          delta.finalCoverage.duplicateBytes -
          delta.prior.duplicateBytes,
      reason: delta.evidence,
    );
    expect(
      delta.finalCoverage.uniqueBytes,
      greaterThanOrEqualTo(delta.prior.uniqueBytes),
      reason: delta.evidence,
    );
    if (delta.after.requests == delta.before.requests) {
      _expectStableCoverage(delta);
      return;
    }
    expect(delta.prior.isComplete, isFalse, reason: delta.evidence);
    expect(
      delta.finalCoverage.uniqueBytes,
      greaterThan(delta.prior.uniqueBytes),
      reason: delta.evidence,
    );
  }

  void _expectStableCoverage(_OriginCoverageDelta delta) {
    expect(
      delta.finalCoverage.networkBytes,
      delta.prior.networkBytes,
      reason: delta.evidence,
    );
    expect(
      delta.finalCoverage.uniqueBytes,
      delta.prior.uniqueBytes,
      reason: delta.evidence,
    );
    expect(
      delta.finalCoverage.duplicateBytes,
      delta.prior.duplicateBytes,
      reason: delta.evidence,
    );
    expect(
      delta.finalCoverage.missingRanges,
      delta.prior.missingRanges,
      reason: delta.evidence,
    );
  }

  void _expectReplayRequests(
    List<ProgressiveOriginRequest> added,
    ProgressiveOriginCoverage prior,
    String evidence,
  ) {
    expect(added.where((request) => request.method == 'HEAD'), isEmpty);
    expect(
      added.every(
        (request) => request.method == 'GET' && request.servedBytes > 0,
      ),
      isTrue,
      reason: evidence,
    );
    if (added.isEmpty) return;
    expect(prior.isComplete, isFalse, reason: evidence);
    expect(added, hasLength(prior.missingRanges.length), reason: evidence);
    expect(
      progressiveReplayCrossesMissingFrontiers(added, prior.missingRanges),
      isTrue,
      reason: evidence,
    );
  }
}
