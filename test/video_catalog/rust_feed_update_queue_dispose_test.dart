import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

void main() {
  test('disposing a feed queue closes its active watcher', () async {
    final upstream = StreamController<FfiFeedUpdate>(sync: true);
    final queue = RustFeedUpdateQueue(upstream.stream);
    final finished = expectLater(queue.watch(), emitsDone);

    await queue.dispose();

    await finished;
    await upstream.close();
  });
}
