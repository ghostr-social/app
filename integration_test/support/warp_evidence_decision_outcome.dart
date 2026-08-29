part of 'warp_evidence_models.dart';

final class WarpDecisionOutcome {
  const WarpDecisionOutcome({
    required this.status,
    required this.bytes,
    required this.elapsedMs,
    required this.failureClass,
    required this.claimRefusal,
  });

  factory WarpDecisionOutcome.fromJson(Map<String, Object?> json) =>
      WarpDecisionOutcome(
        status: _warpString(json, 'status'),
        bytes: json.containsKey('bytes') ? _warpInt(json, 'bytes') : null,
        elapsedMs: json.containsKey('elapsed_ms')
            ? _warpInt(json, 'elapsed_ms')
            : null,
        failureClass: json.containsKey('class')
            ? _warpString(json, 'class')
            : null,
        claimRefusal: json.containsKey('reason')
            ? _warpString(json, 'reason')
            : null,
      );

  final String status;
  final int? bytes;
  final int? elapsedMs;
  final String? failureClass;
  final String? claimRefusal;
}
