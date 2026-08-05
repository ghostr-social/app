import 'dart:async';

import 'package:ghostr/src/rust/api/feed_types.dart';

/// Buffers one feed's snapshot stream so the pull-shaped source can
/// await "the next revision" with a deadline, even when snapshots land
/// before anyone waits for them.
///
/// Only the newest snapshot is held. Rust publishes a full ordered list
/// per revision and an open feed keeps publishing all session long
/// (rust/src/api/feed_updates_stream.rs), so a queue of snapshots would
/// grow with the session while every entry but the last is already
/// contained in it.
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
  FfiFeedUpdate? _pending;
  final Set<Completer<FfiFeedUpdate?>> _waiters = {};
  (Object, StackTrace)? _error;
  bool _done = false;

  /// Whether the stream has ended or died: nothing more will arrive and
  /// the feed behind it needs reopening.
  bool get isFinished => _done;

  /// The next snapshot; null when the stream has ended or [timeout]
  /// passes first. Stream errors surface here, once.
  Future<FfiFeedUpdate?> next(Duration timeout) {
    if (_pending case final pending?) {
      _pending = null;
      return Future.value(pending);
    }
    if (_error case (final error, final stackTrace)) {
      _error = null;
      return Future.error(error, stackTrace);
    }
    if (_done) return Future.value(null);
    return _awaited(timeout);
  }

  /// The newest snapshot that landed while nobody was waiting, if any.
  /// Never waits: this is how a returning caller picks up the pages the
  /// engine filed in the meantime.
  FfiFeedUpdate? drain() {
    final pending = _pending;
    _pending = null;
    return pending;
  }

  Future<void> dispose() {
    _completeAll(null);
    return _subscription.cancel();
  }

  Future<FfiFeedUpdate?> _awaited(Duration timeout) {
    final waiter = Completer<FfiFeedUpdate?>();
    _waiters.add(waiter);
    final deadline = Timer(timeout, () => _complete(waiter, null));
    return waiter.future.whenComplete(() {
      deadline.cancel();
      _waiters.remove(waiter);
    });
  }

  void _add(FfiFeedUpdate update) {
    if (_waiters.isEmpty) {
      _pending = update;
    } else {
      _completeAll(update);
    }
  }

  /// `cancelOnError` ends the subscription with the failure, so the
  /// feed is finished either way — the error is reported first.
  void _fail(Object error, StackTrace stackTrace) {
    _done = true;
    if (_waiters.isEmpty) {
      _error = (error, stackTrace);
    } else {
      final waiters = _takeWaiters();
      for (final waiter in waiters) {
        waiter.completeError(error, stackTrace);
      }
    }
  }

  void _finish() {
    _done = true;
    _completeAll(null);
  }

  void _complete(Completer<FfiFeedUpdate?> waiter, FfiFeedUpdate? update) {
    if (_waiters.remove(waiter) && !waiter.isCompleted) waiter.complete(update);
  }

  void _completeAll(FfiFeedUpdate? update) {
    for (final waiter in _takeWaiters()) {
      if (!waiter.isCompleted) waiter.complete(update);
    }
  }

  List<Completer<FfiFeedUpdate?>> _takeWaiters() {
    final waiters = _waiters.toList(growable: false);
    _waiters.clear();
    return waiters;
  }
}
