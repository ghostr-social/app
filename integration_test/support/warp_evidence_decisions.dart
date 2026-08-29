part of 'warp_evidence_models.dart';

final class WarpDecisionEvidence {
  const WarpDecisionEvidence(this.records);

  factory WarpDecisionEvidence.parse(String encoded) {
    final root = _warpObject(jsonDecode(encoded), r'$');
    _warpSchema(root);
    final decisions = _warpChild(root, 'decisions');
    return WarpDecisionEvidence(
      _warpList(decisions, 'records')
          .map(
            (item) => WarpDecisionRecord.fromJson(
              _warpObject(item, 'decision record'),
            ),
          )
          .toList(growable: false),
    );
  }

  final List<WarpDecisionRecord> records;
}

final class WarpExecutedRequest {
  const WarpExecutedRequest({
    required this.postId,
    required this.sourceId,
    required this.start,
    required this.end,
  });

  factory WarpExecutedRequest.fromJson(Map<String, Object?> json) {
    _warpChild(json, 'resources');
    final range = _warpRecordedRequest(_warpChild(json, 'request'));
    return WarpExecutedRequest(
      postId: _warpString(json, 'post_id'),
      sourceId: _warpString(json, 'source_id'),
      start: range.start,
      end: range.end,
    );
  }

  final String postId;
  final String sourceId;
  final int start;
  final int end;
}

WarpDecisionAction? _warpSelected(Map<String, Object?> json) {
  final raw = json['warp_decision'];
  if (raw == null) return null;
  final selected = _warpObject(raw, 'warp_decision')['selected'];
  if (selected == null) return null;
  return WarpDecisionAction.fromJson(_warpObject(selected, 'selected'));
}

WarpExecutedRequest? _warpExecuted(Map<String, Object?> json) {
  final raw = json['executed_request'];
  return raw == null
      ? null
      : WarpExecutedRequest.fromJson(_warpObject(raw, 'executed_request'));
}

({int start, int end}) _warpRecordedRequest(Map<String, Object?> json) {
  final request = _warpString(json, 'request');
  if (request == 'fetch_range') {
    return (
      start: _warpInt(json, 'bytes_start'),
      end: _warpInt(json, 'bytes_end'),
    );
  }
  if (request != 'fetch_whole') {
    throw FormatException('Unknown recorded request: $request');
  }
  final contract = _warpChild(json, 'contract');
  final endField = _warpString(contract, 'contract') == 'exact'
      ? 'expected_bytes'
      : 'maximum_bytes';
  return (start: 0, end: _warpInt(contract, endField));
}
