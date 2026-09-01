part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionOrigin on _WarpLongSessionDriver {
  void _expectOriginBounded() {
    expect(origin.requests, isNotEmpty);
    expect(origin.hadParallelRangedVideos, isTrue);
    expect(origin.maximumConcurrentResponses, lessThanOrEqualTo(4));
    expect(origin.requests.length, lessThanOrEqualTo(160));
    expect(origin.activeIncompleteRequestSequences, isEmpty);
    expect(origin.headsRemainBlocked, isFalse);
    final coverages = _requestedVideoIds().map(origin.coverageFor).toList();
    for (final coverage in coverages) {
      _expectCoverageBounded(coverage);
    }
    expect(
      progressiveReplayCancellationOverlapWithin(
        coverages,
        budgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
    );
  }

  void _expectCoverageBounded(ProgressiveOriginCoverage coverage) {
    expect(coverage.isWithinObject, isTrue);
    expect(coverage.completedDuplicateBytes, 0);
    expect(
      coverage.duplicateBytes,
      coverage.cancellationAttributedDuplicateBytes,
    );
  }

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
