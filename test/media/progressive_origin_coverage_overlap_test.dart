import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('served interval coverage exposes duplicate bytes and a hole', () {
    final first = ProgressiveOriginRequest('GET', '/video.mp4', (
      start: 0,
      end: 60,
    ), startedAt: Duration.zero)..servedBytes = 50;
    final second = ProgressiveOriginRequest('GET', '/video.mp4', (
      start: 40,
      end: 90,
    ), startedAt: Duration.zero)..servedBytes = 50;

    final coverage = ProgressiveOriginCoverage.fromRequests([
      first,
      second,
    ], objectLength: 100);

    expect(coverage.networkBytes, 100);
    expect(coverage.uniqueBytes, 90);
    expect(coverage.duplicateBytes, 10);
    expect(coverage.missingRanges, [(start: 90, end: 100)]);
    expect(coverage.isComplete, isFalse);
  });
}
