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

part 'ffi_playback_observation_mapping.dart';
part 'ffi_playback_presentation_queue.dart';

typedef RustPlaybackReporter =
    Future<void> Function({required FfiPlaybackObservation input});

final class FfiPlaybackTelemetryPort implements PlaybackTelemetryPort {
  FfiPlaybackTelemetryPort({
    RustPlaybackReporter reportPlayback = ffiReportPlayback,
    RustPlaybackPresentationReporter reportPresentation =
        ffiReportPlaybackPresentation,
    PlaybackPresentationClock presentationClock = _defaultPresentationClock,
  }) : _reportPlayback = reportPlayback,
       _presentations = _PlaybackPresentationQueue(
         reportPresentation,
         presentationClock,
       );

  static const _pendingSessionLimit = 2;
  static var _nextGeneration = 0;

  final RustPlaybackReporter _reportPlayback;
  final _PlaybackPresentationQueue _presentations;
  final LinkedHashMap<int, ListQueue<FfiPlaybackObservation>> _pending =
      LinkedHashMap<int, ListQueue<FfiPlaybackObservation>>();
  final _latest = <int, FfiPlaybackObservation>{};
  PlaybackSession? _active;
  Future<void>? _draining;
  FfiPlaybackObservation? _sendingInput;
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
  void presented(PlaybackSession session) {
    if (_active == session) _presentations.send(session);
  }

  @override
  void deactivate(PlaybackSession session) {
    if (_active == session) _active = null;
    _retainTerminalSample(session.generation);
  }

  void _retainTerminalSample(int generation) {
    final queued = _pending[generation];
    final failure =
        queued?.where(_isFailed).lastOrNull ??
        (_latest[generation]?.phase == FfiPlaybackPhase.failed
            ? _latest[generation]
            : null);
    final terminal =
        queued?.where(_isInactive).lastOrNull ??
        _terminalAfter(_latest[generation]);
    if (terminal == null) {
      _pending.remove(generation);
      return;
    }
    final retained = <FfiPlaybackObservation>[];
    if (failure != null && !identical(failure, _sendingInput)) {
      retained.add(failure);
    }
    retained.add(terminal);
    _latest[generation] = terminal;
    _pending[generation] = ListQueue.of(retained);
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
      _sendingInput = input;
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
      _sendingInput = null;
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
        (generation) => _sendingInput?.generation != BigInt.from(generation),
      );
      _pending.remove(discard);
      _latest.remove(discard);
    }
  }
}
