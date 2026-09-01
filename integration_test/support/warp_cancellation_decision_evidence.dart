import 'warp_evidence_models.dart';

int warpZeroByteCancellationSequenceBaseline(
  Iterable<WarpDecisionRecord> records,
) {
  var latest = 0;
  for (final record in records) {
    if (record.outcome.status != 'cancelled') continue;
    if (record.outcome.bytes != 0) continue;
    if (record.sequence > latest) latest = record.sequence;
  }
  return latest;
}

WarpDecisionRecord? warpZeroByteCancellationAfter(
  Iterable<WarpDecisionRecord> records,
  int sequenceBaseline,
) {
  for (final record in records) {
    if (_isNewZeroByteCancellation(record, sequenceBaseline)) return record;
  }
  return null;
}

int warpExactPendingTransferSequence(
  Iterable<WarpDecisionRecord> records, {
  required int actionId,
  required String postId,
  required ({int start, int end}) range,
}) {
  final matched = records
      .where(
        (record) => _isExactPendingTransfer(
          record,
          actionId: actionId,
          postId: postId,
          range: range,
        ),
      )
      .toList();
  if (matched.length == 1) return matched.single.sequence;
  throw StateError(
    'Expected one exact pending transfer, found ${matched.length}.',
  );
}

WarpDecisionRecord? warpZeroByteCancellationForSequence(
  Iterable<WarpDecisionRecord> records,
  int sequence,
) {
  for (final record in records) {
    if (record.sequence != sequence) continue;
    return _isZeroByteCancellation(record) ? record : null;
  }
  return null;
}

bool _isExactPendingTransfer(
  WarpDecisionRecord record, {
  required int actionId,
  required String postId,
  required ({int start, int end}) range,
}) {
  final request = record.executed;
  return record.outcome.status == 'pending' &&
      record.chosenActionId == actionId &&
      request?.postId == postId &&
      request?.start == range.start &&
      request?.end == range.end;
}

bool _isZeroByteCancellation(WarpDecisionRecord record) =>
    record.outcome.status == 'cancelled' && record.outcome.bytes == 0;

bool _isNewZeroByteCancellation(
  WarpDecisionRecord record,
  int sequenceBaseline,
) =>
    record.sequence > sequenceBaseline &&
    record.outcome.status == 'cancelled' &&
    record.outcome.bytes == 0;

String warpDecisionRetentionEvidence(Iterable<WarpDecisionRecord> records) {
  final retained = records.toList();
  if (retained.isEmpty) return 'retained=empty';
  var oldest = retained.first.sequence;
  var latest = oldest;
  for (final record in retained.skip(1)) {
    if (record.sequence < oldest) oldest = record.sequence;
    if (record.sequence > latest) latest = record.sequence;
  }
  return 'retained=$oldest..$latest/${retained.length}';
}
