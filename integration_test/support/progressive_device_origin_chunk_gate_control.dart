part of 'progressive_device_origin.dart';

extension ProgressiveOriginChunkGateControl on ProgressiveDeviceOrigin {
  ProgressiveOriginChunkGate holdAfterChunks(
    Set<String> paths, {
    required int afterChunks,
    Duration timeout = const Duration(seconds: 2),
  }) {
    if (_chunkGate case final active? when !active.isReleased) {
      throw StateError('A chunk gate is already active.');
    }
    return _chunkGate = ProgressiveOriginChunkGate._(
      paths,
      afterChunks,
      timeout,
    );
  }
}
