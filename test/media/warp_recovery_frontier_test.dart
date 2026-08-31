import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';
import '../../integration_test/support/warp_recovery_frontier.dart';

void main() {
  test('recovery starts after the exact ordered-ready roster prefix', () {
    final frontier = warpRecoveryFrontier((
      futureRosterPaths: const ['/next', '/third', '/fourth', '/fifth'],
      candidatePaths: const ['/next', '/third', '/fourth'],
      projectedPaths: const ['/next', '/third'],
      candidateStates: const [
        WarpReserveCandidateState.ready,
        WarpReserveCandidateState.ready,
        WarpReserveCandidateState.preparing,
      ],
      orderedReady: 2,
      candidateCount: 3,
      minimumReadyDepth: 2,
    ));

    expect(frontier.readyDepth, 2);
    expect(frontier.activationPath, '/third');
    expect(frontier.firstUnreadyPath, '/fourth');
    expect(frontier.transitionPaths, {'/fourth'});
    expect(
      () => warpRecoveryFrontier((
        futureRosterPaths: const ['/next', '/third', '/fourth', '/fifth'],
        candidatePaths: const ['/next', '/third', '/fourth'],
        projectedPaths: const ['/third', '/fourth'],
        candidateStates: const [
          WarpReserveCandidateState.ready,
          WarpReserveCandidateState.ready,
          WarpReserveCandidateState.preparing,
        ],
        orderedReady: 2,
        candidateCount: 3,
        minimumReadyDepth: 2,
      )),
      throwsStateError,
    );
  });
}
