import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_cancellation_decision_evidence.dart';
import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('accepts only a new zero-byte cancelled decision', () {
    final records = [
      _record(4, status: 'cancelled', bytes: 0),
      _record(6, status: 'cancelled', bytes: 32),
      _record(7, status: 'succeeded', bytes: 0),
      _record(8, status: 'cancelled', bytes: 0),
    ];

    final baseline = warpZeroByteCancellationSequenceBaseline(records.take(3));
    final found = warpZeroByteCancellationAfter(records, baseline);

    expect(baseline, 4);
    expect(found?.sequence, 8);
    expect(warpZeroByteCancellationAfter(records.take(3), 4), isNull);
    expect(warpDecisionRetentionEvidence(records), 'retained=4..8/4');
  });

  test('correlates one pending transfer among parallel work exactly', () {
    final pending = [
      _record(13, status: 'pending', bytes: 0, postId: 'target', actionId: 7),
      _record(16, status: 'pending', bytes: 0, postId: 'decoy', actionId: 7),
      _record(
        18,
        status: 'pending',
        bytes: 0,
        postId: 'target',
        actionId: 9,
        start: 65_536,
        end: 131_072,
      ),
    ];
    final sequence = warpExactPendingTransferSequence(
      pending,
      actionId: 7,
      postId: 'target',
      range: (start: 0, end: 65_536),
    );
    final resolved = [
      _record(
        13,
        status: 'cancelled',
        bytes: 0,
        postId: 'privacy-pseudonym',
        actionId: 7,
      ),
    ];

    expect(sequence, 13);
    expect(
      warpZeroByteCancellationForSequence(resolved, sequence)?.sequence,
      13,
    );
  });
}

WarpDecisionRecord _record(
  int sequence, {
  required String status,
  required int bytes,
  String? postId,
  int? actionId,
  int start = 0,
  int end = 65_536,
}) => WarpDecisionRecord(
  sequence: sequence,
  chosenActionId: actionId,
  outcome: WarpDecisionOutcome(
    status: status,
    bytes: bytes,
    elapsedMs: 1,
    failureClass: null,
    claimRefusal: null,
  ),
  selected: null,
  executed: postId == null
      ? null
      : WarpExecutedRequest(
          postId: postId,
          sourceId: 'fixture',
          start: start,
          end: end,
        ),
  observedAtMs: sequence,
  networkThroughputBps: 0,
  plannerNetworkRateBytesPerSecond: null,
);
