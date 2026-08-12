part of 'ffi_feed_focus_port.dart';

const _maximumActiveFocusWrites = 2;

final class _FocusWrite {
  const _FocusWrite(this.window, this.watched, this.generation);

  final _FfiFocusWindow window;
  final Duration watched;
  final BigInt generation;
}

final class _FocusWriteScheduler {
  var _active = 0;
  _FocusWrite? _pending;

  void schedule(_FocusWrite work, Future<void> Function(_FocusWrite) send) {
    if (_active < _maximumActiveFocusWrites) {
      _start(work, send);
      return;
    }
    _pending = work;
  }

  void _start(_FocusWrite work, Future<void> Function(_FocusWrite) send) {
    _active += 1;
    unawaited(send(work).whenComplete(() => _finished(send)));
  }

  void _finished(Future<void> Function(_FocusWrite) send) {
    _active -= 1;
    final pending = _pending;
    _pending = null;
    if (pending != null) _start(pending, send);
  }
}
