part of 'progressive_device_origin.dart';

extension ProgressiveOriginBandwidthTriggerControl on ProgressiveDeviceOrigin {
  ProgressiveOriginBandwidthTrigger armBandwidthChangeAfterNextConfirmedChunk(
    Set<String> paths, {
    required int bandwidthKbps,
    Duration timeout = const Duration(seconds: 10),
  }) {
    if (_pacing.current == null) {
      throw StateError('The origin does not use shared bandwidth.');
    }
    if (_bandwidthTrigger case final active? when !active.isReached) {
      throw StateError('A bandwidth trigger is already active.');
    }
    final trigger = ProgressiveOriginBandwidthTrigger._(
      paths,
      () => setBandwidthKbps(bandwidthKbps),
      timeout,
    );
    return _bandwidthTrigger = trigger;
  }
}
