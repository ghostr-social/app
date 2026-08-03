import 'dart:async';

class KeyedSerialTaskQueue {
  final Map<Object, Future<void>> _tails = <Object, Future<void>>{};

  Future<T> run<T>(Object key, Future<T> Function() operation) async {
    final previous = _tails[key] ?? Future<void>.value();
    final release = Completer<void>();
    final tail = release.future;
    _tails[key] = tail;
    await previous;
    try {
      return await operation();
    } finally {
      release.complete();
      if (identical(_tails[key], tail)) _tails.remove(key);
    }
  }
}
