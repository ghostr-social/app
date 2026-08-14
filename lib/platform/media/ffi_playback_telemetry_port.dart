import 'dart:async';
import 'dart:collection';
import 'dart:developer';

import 'package:ghostr/core/media/playback_delivery_id.dart';
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
  static var _nextGeneration = 0;

  final RustPlaybackReporter _reportPlayback;
  final LinkedHashMap<int, ListQueue<FfiPlaybackObservation>> _pending =
      LinkedHashMap<int, ListQueue<FfiPlaybackObservation>>();
  final _latest = <int, FfiPlaybackObservation>{};
  PlaybackSession? _active;
  Future<void>? _draining;
  int? _sendingGeneration;
  var _nextSequence = 0;

  @override
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  ) {
    return PlaybackSession(videoId, deliveryId, ++_nextGeneration);
  }

  @override
  void activate(PlaybackSession session) {
    final previous = _active;
    if (previous != null && previous != session) {
      _retainTerminalSample(previous.generation);
    }
    _active = session;
    _nextSequence = 0;
  }

  @override
  void report(PlaybackObservation observation) {
    if (_active != observation.session) return;
    final input = _mapObservation(observation, ++_nextSequence);
    _latest[observation.session.generation] = input;
    _enqueue(observation.session.generation, input);
    _boundPendingSessions();
    _draining ??= _drain();
  }

  @override
  void deactivate(PlaybackSession session) {
    if (_active == session) _active = null;
    _retainTerminalSample(session.generation);
  }

  void _retainTerminalSample(int generation) {
    final queued = _pending[generation]?.where(_isInactive).lastOrNull;
    if (queued != null) {
      _replaceQueue(generation, queued);
      return;
    }
    _pending.remove(generation);
    final latest = _latest[generation];
    if (latest == null || _isInactive(latest)) return;
    _queueTerminal(generation, _inactiveAfter(latest));
  }

  void _replaceQueue(int generation, FfiPlaybackObservation terminal) {
    _pending[generation] = ListQueue.of([terminal]);
  }

  void _queueTerminal(int generation, FfiPlaybackObservation terminal) {
    _latest[generation] = terminal;
    _replaceQueue(generation, terminal);
    _draining ??= _drain();
  }

  void _enqueue(int generation, FfiPlaybackObservation input) {
    final queue = _pending.putIfAbsent(generation, ListQueue.new);
    if (queue.isNotEmpty && queue.last.phase == input.phase) {
      queue.removeLast();
    }
    queue.add(input);
  }

  Future<void> _drain() async {
    while (_pending.isNotEmpty) {
      final generation = _pending.keys.first;
      final input = _takeNext(generation);
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
      if (_isInactive(input) && identical(_latest[generation], input)) {
        _latest.remove(generation);
      }
    }
    _draining = null;
  }

  FfiPlaybackObservation _takeNext(int generation) {
    final queue = _pending[generation]!;
    final input = queue.removeFirst();
    if (queue.isEmpty) _pending.remove(generation);
    return input;
  }

  void _boundPendingSessions() {
    while (_pending.length > _pendingSessionLimit) {
      final discard = _pending.keys.firstWhere(
        (generation) => generation != _sendingGeneration,
      );
      _pending.remove(discard);
      _latest.remove(discard);
    }
  }
}

bool _needsInactiveFollowUp(FfiPlaybackObservation input) {
  return input.phase != FfiPlaybackPhase.inactive;
}

bool _isInactive(FfiPlaybackObservation input) {
  return input.phase == FfiPlaybackPhase.inactive;
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
    PlaybackPhase.inactive => FfiPlaybackPhase.inactive,
  };
}
