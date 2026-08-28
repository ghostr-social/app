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
    for (final entry in before.entries) {
      _expectReplayOriginUse(entry.key, entry.value, after[entry.key]!);
    }
  }

  void _expectReplayOriginUse(
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
        '$id beforeMissing=${prior.missingRanges} '
        'afterMissing=${finalCoverage.missingRanges}; '
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
  }

  void _expectCoverageIntegrity(
    ProgressiveOriginCoverage prior,
    ProgressiveOriginCoverage finalCoverage,
    String evidence,
  ) {
    expect(prior.isWithinObject, isTrue, reason: evidence);
    expect(prior.duplicateBytes, 0, reason: evidence);
    expect(finalCoverage.isExact, isTrue, reason: evidence);
  }

  void _expectCoverageDelta(_OriginCoverageDelta delta) {
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
      delta.finalCoverage.uniqueBytes - delta.prior.uniqueBytes,
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
    final expected = prior.isComplete ? 0 : prior.missingRanges.length;
    expect(added, hasLength(expected), reason: evidence);
  }
}
