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
  Future<void>? _draining;

  void send(PlaybackSession session) {
    _pending.putIfAbsent(
      session.generation,
      () => FfiPlaybackPresentation(
        postId: session.deliveryId.value,
        generation: BigInt.from(session.generation),
        sequence: BigInt.from(++_nextSequence),
        observedAtMs: BigInt.from(_clock()),
      ),
    );
    while (_pending.length > _pendingLimit) {
      _pending.remove(_pending.keys.first);
    }
    _draining ??= _drain();
  }

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
      if (identical(_pending[generation], input)) _pending.remove(generation);
    }
    _draining = null;
  }
}

int _defaultPresentationClock() => DateTime.now().millisecondsSinceEpoch;
