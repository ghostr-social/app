part of 'progressive_device_origin.dart';

extension ProgressiveOriginPreBodyGateControl on ProgressiveDeviceOrigin {
  ProgressiveOriginPreBodyGate holdBeforeFirstBody(
    Set<String> paths, {
    Duration timeout = const Duration(seconds: 30),
  }) {
    if (_preBodyGate case final active? when !active.isReleased) {
      throw StateError('A pre-body gate is already active.');
    }
    return _preBodyGate = ProgressiveOriginPreBodyGate._(paths, timeout);
  }
}
