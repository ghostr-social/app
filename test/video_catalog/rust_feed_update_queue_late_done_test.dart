import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

void main() {
  test('a watcher started after native completion closes immediately',
      () async {
    final upstream = StreamController<FfiFeedUpdate>(sync: true);
    final queue = RustFeedUpdateQueue(upstream.stream);

    await upstream.close();

    await expectLater(queue.watch(), emitsDone);
    await queue.dispose();
  });
}
