final class LiveMotionWindow {
  Duration? _position;
  Duration _lastChange = Duration.zero;
  Duration longestFreeze = Duration.zero;
  int advances = 0;

  void record(Duration elapsed, Duration? position) {
    if (_position == null) {
      _position = position;
      _lastChange = elapsed;
      return;
    }
    final gap = elapsed - _lastChange;
    if (gap > longestFreeze) longestFreeze = gap;
    if (position != null && position != _position) {
      advances++;
      _lastChange = elapsed;
      _position = position;
    }
  }

  Map<String, Object?> report() => {
    'positionAdvances': advances,
    'longestFreezeMs': longestFreeze.inMilliseconds,
  };
}
