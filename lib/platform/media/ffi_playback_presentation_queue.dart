part of 'ffi_playback_telemetry_port.dart';

typedef RustPlaybackPresentationReporter =
    Future<void> Function({required FfiPlaybackPresentation input});
typedef PlaybackPresentationClock = int Function();

final class _PlaybackPresentationQueue {
  _PlaybackPresentationQueue(this._report, this._clock);

  static const _pendingLimit = 8;
  static var _nextSequence = 0;

  final RustPlaybackPresentationReporter _report;
  final PlaybackPresentationClock _clock;
  final LinkedHashMap<int, FfiPlaybackPresentation> _pending = LinkedHashMap();
  final _settlement = _PlaybackTelemetrySettlement();
  final Map<FfiPlaybackPresentation, int> _tickets = Map.identity();
  Future<void>? _draining;

  void send(PlaybackSession session) {
    if (_pending.containsKey(session.generation)) return;
    final input = _newPresentation(session);
    _pending[session.generation] = input;
    _tickets[input] = _settlement.issue();
    while (_pending.length > _pendingLimit) {
      _resolve(_pending.remove(_pending.keys.first)!);
    }
    _draining ??= _drain();
  }

  Future<void> get settled => _settlement.throughNow();

  Future<void> _drain() async {
    while (_pending.isNotEmpty) {
      final generation = _pending.keys.first;
      final input = _pending[generation]!;
      try {
        await _report(input: input);
      } on Object catch (error, stackTrace) {
        log(
          'Presented-frame telemetry did not reach the delivery engine.',
          name: 'ghostr.video.telemetry',
          error: error,
          stackTrace: stackTrace,
        );
      }
      if (identical(_pending[generation], input)) {
        _pending.remove(generation);
        _resolve(input);
      }
    }
    _draining = null;
  }

  FfiPlaybackPresentation _newPresentation(PlaybackSession session) {
    return FfiPlaybackPresentation(
      postId: session.deliveryId.value,
      generation: BigInt.from(session.generation),
      sequence: BigInt.from(++_nextSequence),
      observedAtMs: BigInt.from(_clock()),
    );
  }

  void _resolve(FfiPlaybackPresentation input) {
    final ticket = _tickets.remove(input);
    if (ticket != null) _settlement.resolve(ticket);
  }
}

int _defaultPresentationClock() => DateTime.now().millisecondsSinceEpoch;
