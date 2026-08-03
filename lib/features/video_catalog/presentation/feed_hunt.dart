import 'dart:async';

/// Quietly re-schedules feed load attempts for as long as the feed sits
/// empty: the delay doubles per consecutive miss so an abandoned screen
/// never hammers the relays, and resets the moment content lands.
final class FeedHunt {
  FeedHunt({
    Duration base = const Duration(seconds: 2),
    Duration cap = const Duration(seconds: 30),
  })  : _base = base,
        _cap = cap;

  final Duration _base;
  final Duration _cap;
  Timer? _timer;
  int _misses = 0;

  /// Schedules [attempt] once, after the current backoff delay.
  void emptied(void Function() attempt) {
    _timer?.cancel();
    _timer = Timer(_delay(), attempt);
    _misses += 1;
  }

  /// Content arrived: stop hunting and restart the backoff from scratch.
  void filled() {
    _timer?.cancel();
    _timer = null;
    _misses = 0;
  }

  void dispose() {
    _timer?.cancel();
    _timer = null;
  }

  Duration _delay() {
    final shift = _misses < 16 ? _misses : 16;
    final delay = _base * (1 << shift);
    return delay > _cap ? _cap : delay;
  }
}
