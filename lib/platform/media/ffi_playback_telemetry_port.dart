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
part 'ffi_playback_settlement.dart';

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
  final _settlement = _PlaybackTelemetrySettlement();
  final LinkedHashMap<int, ListQueue<FfiPlaybackObservation>> _pending =
      LinkedHashMap<int, ListQueue<FfiPlaybackObservation>>();
  final Map<FfiPlaybackObservation, int> _tickets = Map.identity();
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

  Future<void> get settled =>
      Future.wait([_settlement.throughNow(), _presentations.settled]);

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
      _discardPending(generation);
      return;
    }
    final retained = <FfiPlaybackObservation>[];
    if (failure != null && !identical(failure, _sendingInput)) {
      retained.add(failure);
    }
    if (!identical(terminal, _sendingInput)) retained.add(terminal);
    _latest[generation] = terminal;
    _replacePending(generation, retained);
    _draining ??= _drain();
  }

  void _enqueue(int generation, FfiPlaybackObservation input) {
    _track(input);
    final queue = _pending.putIfAbsent(generation, ListQueue.new);
    if (queue.isNotEmpty && queue.last.phase == input.phase) {
      _resolve(queue.removeLast());
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
      _resolve(input);
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
      _discardPending(discard);
      _latest.remove(discard);
    }
  }

  void _replacePending(int generation, List<FfiPlaybackObservation> retained) {
    final previous = _pending[generation];
    if (previous != null) {
      for (final input in previous) {
        if (!retained.any((item) => identical(item, input))) _resolve(input);
      }
    }
    if (retained.isEmpty) {
      _pending.remove(generation);
      return;
    }
    for (final input in retained) {
      _track(input);
    }
    _pending[generation] = ListQueue.of(retained);
  }

  void _discardPending(int generation) {
    final discarded = _pending.remove(generation);
    if (discarded == null) return;
    for (final input in discarded) {
      _resolve(input);
    }
  }

  void _track(FfiPlaybackObservation input) {
    _tickets.putIfAbsent(input, _settlement.issue);
  }

  void _resolve(FfiPlaybackObservation input) {
    final ticket = _tickets.remove(input);
    if (ticket != null) _settlement.resolve(ticket);
  }
}
