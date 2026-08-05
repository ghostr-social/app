import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  const wait = Duration(seconds: 1);

  // The watcher can die between two reads. The failure is buffered like
  // a snapshot, so the next read surfaces it instead of losing it.
  test('surfaces a stream error that landed while nobody was waiting',
      () async {
    final updates = StreamController<FfiFeedUpdate>();
    final queue = RustFeedUpdateQueue(updates.stream);
    updates.addError(StateError('watcher died'));
    await pumpEventQueue();

    await expectLater(queue.next(wait), throwsA(isA<StateError>()));
    await queue.dispose();
  });

  test('reports the buffered snapshot before the error behind it', () async {
    final updates = StreamController<FfiFeedUpdate>();
    final queue = RustFeedUpdateQueue(updates.stream);
    updates.add(rustFeedBaseline());
    updates.addError(StateError('watcher died'));
    await pumpEventQueue();

    expect((await queue.next(wait))?.stage, FfiFeedStage.loading);
    await expectLater(queue.next(wait), throwsA(isA<StateError>()));
    await queue.dispose();
  });
}
