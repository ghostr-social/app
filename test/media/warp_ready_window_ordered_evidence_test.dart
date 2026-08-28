import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_ready_window_acceptance.dart';

void main() {
  test('projected readiness cannot hide an unprepared ordered gap', () {
    expect(_accepted(ordered: 0), isFalse);
    expect(_accepted(ordered: 2), isTrue);
  });
}

bool _accepted({required int ordered}) {
  return warpReadyEvidenceAccepted(
    revision: (preparation: 19, plan: 19),
    sequence: (observation: 30, after: 20),
    readiness: (
      contiguous: 2,
      ordered: ordered,
      minimum: 2,
      target: 4,
      candidateCount: 3,
      goal: WarpReadyWindowGoal.consumeBurst,
    ),
  );
}
