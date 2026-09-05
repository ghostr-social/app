part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionOrigin on _WarpLongSessionDriver {
  void _expectOriginBounded() {
    expect(origin.requests, isNotEmpty);
    expect(origin.maximumConcurrentResponses, greaterThanOrEqualTo(1));
    expect(origin.maximumConcurrentResponses, lessThanOrEqualTo(2));
    expect(origin.requests.length, lessThanOrEqualTo(160));
    expect(origin.activeIncompleteRequestSequences, isEmpty);
    expect(origin.headsRemainBlocked, isFalse);
    final coverages = _requestedVideoIds().map(origin.coverageFor).toList();
    for (final coverage in coverages) {
      _expectCoverageBounded(coverage);
    }
    // Session replay includes reacquisition after cancellation before EOF.
    // The single-request overrun bound is checked at the controlled cancellation.
    debugPrint(
      'WARP_LONG_REPLAY cancellation_overlap_bytes='
      '${_cancellationReplayBytes(coverages)}',
    );
  }

  void _expectCoverageBounded(ProgressiveOriginCoverage coverage) {
    expect(coverage.isWithinObject, isTrue);
    expect(
      coverage.completedDuplicateBytes,
      0,
      reason: _originReplayEvidence([coverage]),
    );
    expect(
      coverage.duplicateBytes,
      coverage.cancellationAttributedDuplicateBytes,
    );
  }

  String _originReplayEvidence(List<ProgressiveOriginCoverage> coverages) {
    final overlap = _cancellationReplayBytes(coverages);
    final requests = origin.requests.map(
      (request) =>
          '${request.startedAt}:${request.path}:${_requestSummary(request)}',
    );
    return 'Cancellation overlap=$overlap; ${requests.join('|')}';
  }

  int _cancellationReplayBytes(List<ProgressiveOriginCoverage> coverages) =>
      coverages.fold<int>(
        0,
        (total, coverage) =>
            total + coverage.cancellationAttributedDuplicateBytes,
      );

  int _canceledRequestCount() => origin.requests
      .where(
        (request) =>
            request.outcome == ProgressiveOriginRequestOutcome.clientCanceled,
      )
      .length;

  Set<String> _requestedVideoIds() => origin.requests
      .map((request) => request.path)
      .where((path) => path.endsWith('.mp4'))
      .map((path) => path.substring(1, path.length - 4))
      .toSet();
}
