import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/device_qoe_targets.dart';
import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('replay permits only bounded cancellation-attributed overlap', () {
    const tailBytes = 64 * 1024;
    const objectLength = deviceCancellationWasteTargetBytes + tailBytes;
    const allowedOverlap = deviceCancellationWasteTargetBytes ~/ 2 + 1;
    final allowed = _coverage([
      _request(0, allowedOverlap, canceled: true),
      _request(0, allowedOverlap),
      _request(allowedOverlap, objectLength),
    ], objectLength);
    final completedOverlap = _coverage([
      _request(0, tailBytes),
      _request(tailBytes - 1, objectLength),
    ], objectLength);
    final overBudget = _coverage([
      _request(0, deviceCancellationWasteTargetBytes + 1, canceled: true),
      _request(0, deviceCancellationWasteTargetBytes + 1),
      _request(deviceCancellationWasteTargetBytes + 1, objectLength),
    ], objectLength);

    expect(allowed.isComplete, isTrue);
    expect(allowed.uniqueBytes, objectLength);
    expect(allowed.completedDuplicateBytes, 0);
    expect(allowed.cancellationAttributedDuplicateBytes, allowedOverlap);
    expect(
      allowed.isReplayCompleteWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
    );
    expect(
      progressiveReplayCancellationOverlapWithin([
        allowed,
        allowed,
      ], budgetBytes: deviceCancellationWasteTargetBytes),
      isFalse,
    );
    expect(
      allowed.hasReplayIntegrityWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
    );
    expect(completedOverlap.completedDuplicateBytes, 1);
    expect(
      completedOverlap.isReplayCompleteWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isFalse,
    );
    expect(
      overBudget.cancellationAttributedDuplicateBytes,
      deviceCancellationWasteTargetBytes + 1,
    );
    expect(
      overBudget.isReplayCompleteWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isFalse,
    );
  });
}

ProgressiveOriginCoverage _coverage(
  List<ProgressiveOriginRequest> requests,
  int objectLength,
) {
  return ProgressiveOriginCoverage.fromRequests(
    requests,
    objectLength: objectLength,
  );
}

ProgressiveOriginRequest _request(int start, int end, {bool canceled = false}) {
  return ProgressiveOriginRequest('GET', '/video.mp4', (
      start: start,
      end: end,
    ), startedAt: Duration.zero)
    ..servedBytes = end - start
    ..outcome = canceled
        ? ProgressiveOriginRequestOutcome.clientCanceled
        : ProgressiveOriginRequestOutcome.completed;
}
