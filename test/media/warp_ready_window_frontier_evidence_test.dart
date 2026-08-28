import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_ready_window_acceptance.dart';

void main() {
  test('recovery requires an explicitly modeled unready candidate', () {
    expect(_accepted(candidateCount: 2), isFalse);
    expect(_accepted(candidateCount: 3), isTrue);
  });
}

bool _accepted({required int candidateCount}) {
  return warpReadyEvidenceAccepted(
    revision: (preparation: 19, plan: 19),
    sequence: (observation: 30, after: 20),
    readiness: (
      contiguous: 2,
      ordered: 2,
      minimum: 2,
      target: 4,
      candidateCount: candidateCount,
      goal: WarpReadyWindowGoal.recoveryFrontier,
    ),
  );
}
