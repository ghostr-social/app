import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('cancelled swipe replay may remain sparse without corrupting bytes', () {
    final initial =
        ProgressiveOriginRequest('GET', '/video.mp4', (
            start: 0,
            end: 108041,
          ), startedAt: Duration.zero)
          ..servedBytes = 108041
          ..outcome = ProgressiveOriginRequestOutcome.completed;
    final replay =
        ProgressiveOriginRequest('GET', '/video.mp4', (
            start: 108041,
            end: 110229,
          ), startedAt: Duration.zero)
          ..servedBytes = 2188
          ..outcome = ProgressiveOriginRequestOutcome.clientCanceled;
    final restarted =
        ProgressiveOriginRequest('GET', '/video.mp4', (
            start: 0,
            end: 2188,
          ), startedAt: Duration.zero)
          ..servedBytes = 2188
          ..outcome = ProgressiveOriginRequestOutcome.clientCanceled;
    final prior = ProgressiveOriginCoverage.fromRequests([
      initial,
    ], objectLength: 293999);
    final coverage = ProgressiveOriginCoverage.fromRequests([
      initial,
      replay,
    ], objectLength: 293999);

    expect(coverage.missingRanges, [(start: 110229, end: 293999)]);
    expect(coverage.networkBytes, 110229);
    expect(coverage.uniqueBytes, greaterThan(prior.uniqueBytes));
    expect(
      progressiveReplayCrossesMissingFrontiers([replay], prior.missingRanges),
      isTrue,
    );
    expect(
      progressiveReplayCrossesMissingFrontiers([
        restarted,
      ], prior.missingRanges),
      isFalse,
    );
    expect(
      coverage.hasReplayIntegrityWithin(cancellationOverlapBudgetBytes: 0),
      isTrue,
    );
    expect(
      coverage.isReplayCompleteWithin(cancellationOverlapBudgetBytes: 0),
      isFalse,
    );
  });
}
