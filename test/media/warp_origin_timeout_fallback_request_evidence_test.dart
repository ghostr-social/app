import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';
import '../../integration_test/support/warp_origin_timeout_fallback_scenario.dart';

void main() {
  test('fallback evidence accepts bounded exact ranged assembly', () {
    const objectLength = 293999;
    final requests = [
      _completedRange(0, 65536),
      _completedRange(65536, 99072),
      _completedRange(99072, 110229),
      _completedRange(110229, 119804),
      _completedRange(119804, objectLength),
    ];

    expect(
      warpOriginTimeoutHasBoundedExactFallback(
        requests,
        objectLength: objectLength,
      ),
      isTrue,
    );
    expect(
      warpOriginTimeoutHasBoundedExactFallback(
        requests.take(1),
        objectLength: objectLength,
      ),
      isFalse,
    );
  });
}

ProgressiveOriginRequest _completedRange(int start, int end) {
  return ProgressiveOriginRequest('GET', '/next-rescue.mp4', (
      start: start,
      end: end,
    ), startedAt: Duration.zero)
    ..servedBytes = end - start
    ..outcome = ProgressiveOriginRequestOutcome.completed;
}
