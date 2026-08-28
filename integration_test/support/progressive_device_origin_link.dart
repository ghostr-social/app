part of 'progressive_device_origin.dart';

final class _ProgressiveOriginPacer {
  _ProgressiveOriginPacer(ProgressiveOriginPacing pacing)
    : _responseDelay = pacing.responseChunkDelay,
      _bandwidthKbps = pacing.bandwidthKbps {
    if (pacing.isShared) _current = _profile(pacing.bandwidthKbps!, const {});
  }

  final _clock = Stopwatch()..start();
  final Duration _responseDelay;
  int? _bandwidthKbps;
  var _generation = 0;
  Future<void> _tail = Future<void>.value();
  ProgressiveOriginLinkProfile? _current;
  final events = <ProgressiveOriginChunkEvent>[];

  ProgressiveOriginLinkProfile? get current => _current;

  Future<_ProgressiveOriginChunkPermit?> acquire(int bytes) {
    if (_bandwidthKbps == null) {
      return Future<_ProgressiveOriginChunkPermit?>.value();
    }
    final released = Completer<void>();
    final previous = _tail;
    _tail = released.future;
    return _serve(previous, released, bytes);
  }

  Future<_ProgressiveOriginChunkPermit> _serve(
    Future<void> previous,
    Completer<void> released,
    int bytes,
  ) async {
    await previous;
    final profile = _current!;
    final started = _clock.elapsed;
    await Future<void>.delayed(_serviceTime(bytes, profile.bandwidthKbps));
    return _ProgressiveOriginChunkPermit(
      profile: profile,
      serviceStartedAt: started,
      release: released.complete,
    );
  }

  ProgressiveOriginChunkEvent record(
    _ProgressiveOriginChunkPermit permit,
    _ProgressiveChunk chunk,
  ) {
    final event = ProgressiveOriginChunkEvent._((
      requestSequence: chunk.requestSequence,
      path: chunk.path,
      start: chunk.start,
      end: chunk.end,
      profileGeneration: permit.profile.generation,
      bandwidthKbps: permit.profile.bandwidthKbps,
      serviceStartedAt: permit.serviceStartedAt,
      sentAt: _clock.elapsed,
      sentAtEpochMs: DateTime.now().millisecondsSinceEpoch,
    ));
    events.add(event);
    return event;
  }

  void confirm(ProgressiveOriginChunkEvent event) =>
      event.confirmedAtEpochMs = DateTime.now().millisecondsSinceEpoch;

  Future<void> afterChunk(bool hasMore) async {
    if (hasMore && _responseDelay > Duration.zero) {
      await Future<void>.delayed(_responseDelay);
    }
  }
}

Duration _serviceTime(int bytes, int bandwidthKbps) {
  final micros = (bytes * 8000 + bandwidthKbps - 1) ~/ bandwidthKbps;
  return Duration(microseconds: micros);
}
