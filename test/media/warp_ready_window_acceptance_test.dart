import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_ready_window_acceptance.dart';

void main() {
  test('a ready burst does not wait for a larger adaptive target', () {
    expect(
      warpReadyWindowAccepted(
        contiguousDepth: 3,
        minimumDepth: 3,
        planTarget: 5,
        goal: WarpReadyWindowGoal.consumeBurst,
      ),
      isTrue,
    );
    expect(
      warpReadyWindowAccepted(
        contiguousDepth: 3,
        minimumDepth: 3,
        planTarget: 5,
        goal: WarpReadyWindowGoal.fillTarget,
      ),
      isFalse,
    );
    expect(
      warpReadyWindowAccepted(
        contiguousDepth: 2,
        minimumDepth: 3,
        planTarget: 2,
        goal: WarpReadyWindowGoal.consumeBurst,
      ),
      isFalse,
    );
    expect(
      warpReadyWindowAccepted(
        contiguousDepth: 5,
        minimumDepth: 3,
        planTarget: 5,
        goal: WarpReadyWindowGoal.fillTarget,
      ),
      isTrue,
    );
  });

  test('a later target cannot certify an earlier readiness snapshot', () {
    expect(
      warpReadyEvidenceAccepted(
        revision: (preparation: 18, plan: 19),
        sequence: (observation: 30, after: 20),
        readiness: (
          contiguous: 4,
          ordered: 4,
          minimum: 3,
          target: 1,
          candidateCount: 4,
          goal: WarpReadyWindowGoal.fillTarget,
        ),
      ),
      isFalse,
    );
    expect(
      warpReadyEvidenceAccepted(
        revision: (preparation: 19, plan: 19),
        sequence: (observation: 30, after: 20),
        readiness: (
          contiguous: 4,
          ordered: 4,
          minimum: 3,
          target: 1,
          candidateCount: 4,
          goal: WarpReadyWindowGoal.fillTarget,
        ),
      ),
      isTrue,
    );
  });

  test('a rejected newest snapshot cannot fall back to stale history', () {
    final history = <WarpEvidenceCursor>[
      (revision: 18, sequence: 30),
      (revision: 19, sequence: 31),
    ];

    expect(
      warpNewestCausalEvidenceIndex(
        history: history,
        afterRevision: 17,
        afterSequence: 20,
      ),
      1,
    );
  });
}
