import 'dart:async';

import 'package:ghostr/src/rust/api/feed_types.dart';

typedef RustFeedObserverStarted = void Function(
  RustFeedUpdateObserver observer,
);
typedef RustFeedObserverCancelled = void Function(
  RustFeedUpdateObserver observer,
);

/// One coalescing feed watcher. A paused listener retains only the newest full
/// snapshot, preserving the native queue's bounded-memory contract.
final class RustFeedUpdateObserver {
  RustFeedUpdateObserver({
    required RustFeedObserverStarted onStarted,
    required RustFeedObserverCancelled onCancelled,
  })  : _onStarted = onStarted,
        _onCancelled = onCancelled {
    _controller = StreamController<FfiFeedUpdate>(
      sync: true,
      onListen: _started,
      onPause: _paused,
      onResume: _resumed,
      onCancel: _cancelled,
    );
  }

  final RustFeedObserverStarted _onStarted;
  final RustFeedObserverCancelled _onCancelled;
  late final StreamController<FfiFeedUpdate> _controller;
  BigInt _revision = BigInt.from(-1);
  FfiFeedUpdate? _pending;
  bool _isPaused = false;

  Stream<FfiFeedUpdate> get stream => _controller.stream;

  void publish(FfiFeedUpdate update) {
    if (update.revision <= _revision) return;
    _revision = update.revision;
    if (_isPaused) {
      _pending = update;
    } else {
      _controller.add(update);
    }
  }

  Future<void> fail(Object error, StackTrace stackTrace) async {
    _flush();
    _controller.addError(error, stackTrace);
    await _controller.close();
  }

  Future<void> close() async {
    _flush();
    await _controller.close();
  }

  void _started() => _onStarted(this);

  void _paused() => _isPaused = true;

  void _resumed() {
    _isPaused = false;
    _flush();
  }

  void _cancelled() => _onCancelled(this);

  void _flush() {
    final pending = _pending;
    _pending = null;
    if (pending != null) _controller.add(pending);
  }
}
