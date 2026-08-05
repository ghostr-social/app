import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  test('a paused watcher retains only the newest full snapshot', () async {
    final upstream = StreamController<FfiFeedUpdate>(sync: true);
    final queue = RustFeedUpdateQueue(upstream.stream);
    final revisions = <BigInt>[];
    final watch =
        queue.watch().listen((update) => revisions.add(update.revision));
    watch.pause();

    for (var revision = 1; revision <= 500; revision++) {
      upstream.add(rustFeedUpdate(revision: revision));
    }
    watch.resume();
    await pumpEventQueue();

    expect(revisions, [BigInt.from(500)]);
    await watch.cancel();
    await queue.dispose();
    await upstream.close();
  });
}
