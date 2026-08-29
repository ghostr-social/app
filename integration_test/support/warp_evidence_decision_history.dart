part of 'warp_evidence_models.dart';

extension WarpMaterialDecisionHistory on WarpDecisionEvidence {
  Iterable<WarpDecisionRecord> get materialRecords =>
      records.where((record) => !record._isExactNoop);
}

extension on WarpDecisionRecord {
  bool get _isExactNoop => !hasWarpDecision && _hasNoAction && _isEmptySuccess;

  bool get _hasNoAction =>
      chosenActionId == null && selected == null && executed == null;

  bool get _isEmptySuccess =>
      outcome.status == 'succeeded' &&
      outcome.bytes == 0 &&
      outcome.elapsedMs == 0;
}
