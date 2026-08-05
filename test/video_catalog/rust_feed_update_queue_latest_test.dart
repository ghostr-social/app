import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  const wait = Duration(seconds: 1);

  // A feed that stays open publishes a full snapshot per revision all
  // session long. Every snapshot carries the whole list, so keeping the
  // newest is keeping everything — buffering one per revision would
  // grow with the session instead.
  test('keeps only the newest snapshot while nobody is waiting', () async {
    final updates = StreamController<FfiFeedUpdate>();
    final queue = RustFeedUpdateQueue(updates.stream);
    updates
      ..add(rustFeedUpdate(revision: 1))
      ..add(rustFeedUpdate(revision: 2))
      ..add(rustFeedUpdate(revision: 3));
    await pumpEventQueue();

    expect((await queue.next(wait))?.revision, BigInt.from(3));
    expect(await queue.next(wait), isNull, reason: 'nothing is left behind');
    await queue.dispose();
  });

  test('hands the newest snapshot to a caller that will not wait', () async {
    final updates = StreamController<FfiFeedUpdate>();
    final queue = RustFeedUpdateQueue(updates.stream);
    updates
      ..add(rustFeedUpdate(revision: 4))
      ..add(rustFeedUpdate(revision: 5));
    await pumpEventQueue();

    expect(queue.drain()?.revision, BigInt.from(5));
    expect(queue.drain(), isNull);
    await queue.dispose();
  });
}
