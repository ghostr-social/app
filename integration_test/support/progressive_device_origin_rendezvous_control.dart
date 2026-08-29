part of 'progressive_device_origin.dart';

extension ProgressiveDeviceOriginRendezvousControl on ProgressiveDeviceOrigin {
  ProgressiveOriginFirstChunkRendezvous rendezvousFirstChunks(
    Set<String> paths, {
    Duration timeout = const Duration(seconds: 2),
  }) => _installRendezvous(
    paths,
    timeout,
    activated: true,
    blocksFirstChunks: true,
  );

  ProgressiveOriginFirstChunkRendezvous stageFirstChunks(
    Set<String> paths, {
    Duration timeout = const Duration(seconds: 2),
  }) => _installRendezvous(
    paths,
    timeout,
    activated: false,
    blocksFirstChunks: false,
  );

  ProgressiveOriginFirstChunkRendezvous _installRendezvous(
    Set<String> paths,
    Duration timeout, {
    required bool activated,
    required bool blocksFirstChunks,
  }) {
    final active = _firstChunkRendezvous;
    if (active != null && !active.isReleased) {
      throw StateError('A first-chunk rendezvous is already active.');
    }
    final rendezvous = ProgressiveOriginFirstChunkRendezvous._(
      paths,
      timeout,
      activated,
      blocksFirstChunks,
    );
    _firstChunkRendezvous = rendezvous;
    return rendezvous;
  }
}
