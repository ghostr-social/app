part of 'ffi_playback_telemetry_port.dart';

bool _isInactive(FfiPlaybackObservation input) {
  return input.phase == FfiPlaybackPhase.inactive;
}

bool _isFailed(FfiPlaybackObservation input) {
  return input.phase == FfiPlaybackPhase.failed;
}

FfiPlaybackObservation? _terminalAfter(FfiPlaybackObservation? input) {
  if (input == null) return null;
  return _isInactive(input) ? input : _inactiveAfter(input);
}

FfiPlaybackObservation _inactiveAfter(FfiPlaybackObservation input) {
  return FfiPlaybackObservation(
    postId: input.postId,
    generation: input.generation,
    sequence: input.sequence + BigInt.one,
    phase: FfiPlaybackPhase.inactive,
    positionMs: input.positionMs,
    bufferedExtentMs: input.bufferedExtentMs,
    playbackRateMilli: input.playbackRateMilli,
  );
}

FfiPlaybackObservation _mapObservation(
  PlaybackObservation observation,
  int sequence,
) {
  return FfiPlaybackObservation(
    postId: observation.session.deliveryId.value,
    generation: BigInt.from(observation.session.generation),
    sequence: BigInt.from(sequence),
    phase: _mapPhase(observation.phase),
    positionMs: BigInt.from(observation.position.inMilliseconds),
    bufferedExtentMs: BigInt.from(observation.bufferedExtent.inMilliseconds),
    playbackRateMilli: (observation.playbackRate * 1000).round(),
  );
}

FfiPlaybackPhase _mapPhase(PlaybackPhase phase) {
  return switch (phase) {
    PlaybackPhase.starting => FfiPlaybackPhase.starting,
    PlaybackPhase.playing => FfiPlaybackPhase.playing,
    PlaybackPhase.networkStalled => FfiPlaybackPhase.networkStalled,
    PlaybackPhase.paused => FfiPlaybackPhase.paused,
    PlaybackPhase.ended => FfiPlaybackPhase.ended,
    PlaybackPhase.failed => FfiPlaybackPhase.failed,
    PlaybackPhase.inactive => FfiPlaybackPhase.inactive,
  };
}
