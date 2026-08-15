import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/async/keyed_serial_task_queue.dart';

void main() {
  test('drain waits for work queued under every key', () async {
    final queue = KeyedSerialTaskQueue();
    final first = Completer<void>();
    final second = Completer<void>();
    final started = <String>[];
    queue.run('a', () async {
      started.add('a');
      await first.future;
    });
    queue.run('b', () async {
      started.add('b');
      await second.future;
    });
    await pumpEventQueue();
    var drained = false;

    final draining = queue.drain()..then((_) => drained = true);
    first.complete();
    await pumpEventQueue();
    expect(drained, isFalse);
    second.complete();
    await draining;

    expect(started, containsAll(['a', 'b']));
    expect(drained, isTrue);
  });
}
