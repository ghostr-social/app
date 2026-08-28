import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('incomplete replay coverage is not exact completion', () {
    final request = ProgressiveOriginRequest('GET', '/video.mp4', (
      start: 0,
      end: 90,
    ), startedAt: Duration.zero)..servedBytes = 90;

    final coverage = ProgressiveOriginCoverage.fromRequests([
      request,
    ], objectLength: 100);

    expect(coverage.isExact, isFalse);
  });
}
