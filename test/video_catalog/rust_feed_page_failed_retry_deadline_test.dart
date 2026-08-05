import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_page_reader.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_update_queue.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/rust_feed_fixtures.dart';

void main() {
  test('failed retry snapshots cannot extend the pull deadline', () {
    fakeAsync((async) {
      const deadline = Duration(seconds: 10);
      var elapsed = Duration.zero;
      final updates = StreamController<FfiFeedUpdate>();
      final reader = RustFeedPageReader(
        RustFeedUpdateQueue(updates.stream),
        deadline: deadline,
        elapsedClock: () => elapsed,
      );
      RustFeedPage? result;

      reader.firstPage().then((page) => result = page);
      async.flushMicrotasks();
      async.elapse(const Duration(seconds: 2));
      elapsed = const Duration(seconds: 2);
      updates.add(rustFeedUpdate(revision: 1, stage: FfiFeedStage.failed));
      async.flushMicrotasks();
      async.elapse(const Duration(seconds: 6));
      elapsed = const Duration(seconds: 8);
      updates.add(rustFeedUpdate(revision: 2, stage: FfiFeedStage.failed));
      async.flushMicrotasks();
      async.elapse(const Duration(seconds: 1));
      expect(result, isNull);

      elapsed = deadline;
      async.elapse(const Duration(seconds: 1));
      expect(result?.posts, isEmpty);
      unawaited(updates.close());
    });
  });
}
