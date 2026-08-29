part of 'warp_evidence_models.dart';

List<WarpPlanTransfer> _warpTransfers(
  Map<String, Object?> json,
  String field,
) => _warpList(json, field)
    .map((item) {
      final value = _warpObject(item, field);
      final request = _warpRequest(_warpChild(value, 'request'));
      return WarpPlanTransfer((
        postId: _warpString(value, 'post'),
        sourceId: _warpString(value, 'source'),
        requestKind: request.kind,
        start: request.start,
        end: request.end,
        reason: _warpString(value, 'reason'),
        actionId: value.containsKey('action_id')
            ? _warpInt(value, 'action_id')
            : null,
        expectedDeliveryMs: _warpInt(
          _warpChild(value, 'utility'),
          'expected_delivery_ms',
        ),
      ));
    })
    .toList(growable: false);

({int start, int end, WarpTransferRequestKind kind}) _warpRequest(
  Map<String, Object?> json,
) {
  if (json['FetchRange'] case final Object? value?) {
    final range = _warpChild(_warpObject(value, 'FetchRange'), 'bytes');
    return (
      start: _warpInt(range, 'start'),
      end: _warpInt(range, 'end'),
      kind: WarpTransferRequestKind.range,
    );
  }
  final whole = _warpChild(json, 'FetchWhole');
  final contract = _warpChild(whole, 'contract');
  final variant = _warpObject(contract.values.single, 'contract');
  final end = variant.values.single;
  if (end is! int) throw const FormatException('Invalid whole request.');
  return (start: 0, end: end, kind: WarpTransferRequestKind.whole);
}

String _warpVariantName(Object? value) {
  if (value is String && value.isNotEmpty) return value;
  final variant = _warpObject(value, 'variant');
  if (variant.length != 1) throw const FormatException('Invalid variant.');
  return variant.keys.single;
}
