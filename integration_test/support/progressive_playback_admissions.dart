import 'package:ghostr/src/rust/api/playback_control.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

final class ProgressivePlaybackAdmissionProbe {
  const ProgressivePlaybackAdmissionProbe._(this._baseline);

  static Future<ProgressivePlaybackAdmissionProbe> capture() async {
    return ProgressivePlaybackAdmissionProbe._(
      await ffiPlaybackAdmissionSnapshot(),
    );
  }

  final FfiPlaybackAdmissionSnapshot _baseline;

  Future<ProgressivePlaybackAdmissions> delta() async {
    final current = await ffiPlaybackAdmissionSnapshot();
    return ProgressivePlaybackAdmissions(
      accepted: current.accepted - _baseline.accepted,
      inactiveDelivery: current.inactiveDelivery - _baseline.inactiveDelivery,
      staleSession: current.staleSession - _baseline.staleSession,
      staleSequence: current.staleSequence - _baseline.staleSequence,
      lastAcceptedDeliveryId: current.lastAcceptedDeliveryId,
    );
  }
}

final class ProgressivePlaybackAdmissions {
  const ProgressivePlaybackAdmissions({
    required this.accepted,
    required this.inactiveDelivery,
    required this.staleSession,
    required this.staleSequence,
    required this.lastAcceptedDeliveryId,
  });

  final BigInt accepted;
  final BigInt inactiveDelivery;
  final BigInt staleSession;
  final BigInt staleSequence;
  final String? lastAcceptedDeliveryId;
}
