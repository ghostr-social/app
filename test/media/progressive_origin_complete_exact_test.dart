import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('complete non-overlapping origin coverage is exact', () {
    final request = ProgressiveOriginRequest('GET', '/video.mp4', (
      start: 0,
      end: 100,
    ), startedAt: Duration.zero)..servedBytes = 100;

    final coverage = ProgressiveOriginCoverage.fromRequests([
      request,
    ], objectLength: 100);

    expect(coverage.isExact, isTrue);
  });
}
