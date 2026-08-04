import 'dart:async';

import 'package:ghostr/src/rust/api/feed_types.dart';

/// Buffers one feed's snapshot stream so the pull-shaped source can
/// await "the next revision" with a deadline, even when snapshots land
/// before anyone waits for them.
final class RustFeedUpdateQueue {
  RustFeedUpdateQueue(Stream<FfiFeedUpdate> updates) {
    _subscription = updates.listen(
      _add,
      onError: _fail,
      onDone: _finish,
      cancelOnError: true,
    );
  }

  late final StreamSubscription<FfiFeedUpdate> _subscription;
  final List<FfiFeedUpdate> _buffered = <FfiFeedUpdate>[];
  Completer<FfiFeedUpdate?>? _waiter;
  (Object, StackTrace)? _error;
  bool _done = false;

  /// The next snapshot; null when the stream has ended or [timeout]
  /// passes first. Stream errors surface here, once.
  Future<FfiFeedUpdate?> next(Duration timeout) {
    if (_buffered.isNotEmpty) return Future.value(_buffered.removeAt(0));
    if (_error case (final error, final stackTrace)) {
      _error = null;
      return Future.error(error, stackTrace);
    }
    if (_done) return Future.value(null);
    return _awaited(timeout);
  }

  Future<void> dispose() {
    _complete(null);
    return _subscription.cancel();
  }

  Future<FfiFeedUpdate?> _awaited(Duration timeout) {
    final waiter = Completer<FfiFeedUpdate?>();
    _waiter = waiter;
    final deadline = Timer(timeout, () => _complete(null));
    return waiter.future.whenComplete(deadline.cancel);
  }

  void _add(FfiFeedUpdate update) {
    if (_waiter == null) {
      _buffered.add(update);
    } else {
      _complete(update);
    }
  }

  void _fail(Object error, StackTrace stackTrace) {
    final waiter = _waiter;
    _waiter = null;
    if (waiter == null) {
      _error = (error, stackTrace);
    } else {
      waiter.completeError(error, stackTrace);
    }
  }

  void _finish() {
    _done = true;
    _complete(null);
  }

  void _complete(FfiFeedUpdate? update) {
    final waiter = _waiter;
    _waiter = null;
    if (waiter != null && !waiter.isCompleted) waiter.complete(update);
  }
}
