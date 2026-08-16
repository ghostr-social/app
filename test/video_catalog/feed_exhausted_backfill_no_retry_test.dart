import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill_retry.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an exhausted older-page cursor schedules no retry timer', () {
    fakeAsync((clock) {
      final feed = FakeVideoCatalogRepository(
        forYouFeed: [samplePost(id: 'initial')],
      )..olderFeedPages.add([samplePost(id: 'older')]);
      final cubit = FeedCubit(
        FeedDependencies(feed: feed, engagement: feed),
        backfillRetry: FeedBackfillRetry(delays: const [Duration(seconds: 1)]),
      );

      unawaited(cubit.load());
      clock.flushMicrotasks();

      expect((cubit.state as FeedLoaded).posts, hasLength(2));
      expect(clock.nonPeriodicTimerCount, 0);
      unawaited(cubit.close());
      clock.flushMicrotasks();
    });
  });
}
