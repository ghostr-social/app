part of 'ffi_playback_telemetry_port.dart';

final class _PlaybackTelemetrySettlement {
  final _pending = <int>{};
  final _barriers = <_PlaybackTelemetryBarrier>[];
  var _latestTicket = 0;

  int issue() {
    final ticket = ++_latestTicket;
    _pending.add(ticket);
    return ticket;
  }

  Future<void> throughNow() {
    final target = _latestTicket;
    if (!_hasPendingThrough(target)) return Future.value();
    final completer = Completer<void>();
    _barriers.add(_PlaybackTelemetryBarrier(target, completer));
    return completer.future;
  }

  void resolve(int ticket) {
    if (!_pending.remove(ticket)) return;
    _completeReadyBarriers();
  }

  bool _hasPendingThrough(int target) {
    return _pending.any((ticket) => ticket <= target);
  }

  void _completeReadyBarriers() {
    final ready = _barriers
        .where((barrier) => !_hasPendingThrough(barrier.target))
        .toList();
    _barriers.removeWhere(ready.contains);
    for (final barrier in ready) {
      barrier.completer.complete();
    }
  }
}

final class _PlaybackTelemetryBarrier {
  const _PlaybackTelemetryBarrier(this.target, this.completer);

  final int target;
  final Completer<void> completer;
}
