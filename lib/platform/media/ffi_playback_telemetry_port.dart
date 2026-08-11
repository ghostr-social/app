import 'dart:async';
import 'dart:collection';
import 'dart:developer';

import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_control.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

typedef RustPlaybackReporter =
    Future<void> Function({required FfiPlaybackObservation input});

final class FfiPlaybackTelemetryPort implements PlaybackTelemetryPort {
  FfiPlaybackTelemetryPort({
    RustPlaybackReporter reportPlayback = ffiReportPlayback,
  }) : _reportPlayback = reportPlayback;

  static const _pendingSessionLimit = 2;

  final RustPlaybackReporter _reportPlayback;
  final LinkedHashMap<int, FfiPlaybackObservation> _pending =
      LinkedHashMap<int, FfiPlaybackObservation>();
  PlaybackSession? _active;
  Future<void>? _draining;
  int? _sendingGeneration;
  var _nextGeneration = 0;
  var _nextSequence = 0;

  @override
  PlaybackSession activate(PlaybackVideoId videoId) {
    final session = PlaybackSession(videoId, ++_nextGeneration);
    _active = session;
    _nextSequence = 0;
    return session;
  }

  @override
  void report(PlaybackObservation observation) {
    if (_active != observation.session) return;
    final input = _mapObservation(observation, ++_nextSequence);
    _pending[observation.session.generation] = input;
    _boundPendingSessions();
    _draining ??= _drain();
  }

  @override
  void deactivate(PlaybackSession session) {
    if (_active == session) _active = null;
  }

  Future<void> _drain() async {
    while (_pending.isNotEmpty) {
      final generation = _pending.keys.first;
      final input = _pending.remove(generation)!;
      _sendingGeneration = _needsInactiveFollowUp(input) ? generation : null;
      try {
        await _reportPlayback(input: input);
      } on Object catch (error, stackTrace) {
        log(
          'Playback telemetry did not reach the delivery engine.',
          name: 'ghostr.video.telemetry',
          error: error,
          stackTrace: stackTrace,
        );
      }
      _sendingGeneration = null;
    }
    _draining = null;
  }

  void _boundPendingSessions() {
    while (_pending.length > _pendingSessionLimit) {
      final discard = _pending.keys.firstWhere(
        (generation) => generation != _sendingGeneration,
      );
      _pending.remove(discard);
    }
  }
}

bool _needsInactiveFollowUp(FfiPlaybackObservation input) {
  return input.phase != FfiPlaybackPhase.inactive;
}

FfiPlaybackObservation _mapObservation(
  PlaybackObservation observation,
  int sequence,
) {
  return FfiPlaybackObservation(
    postId: observation.videoId.value,
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
    PlaybackPhase.inactive => FfiPlaybackPhase.inactive,
  };
}
