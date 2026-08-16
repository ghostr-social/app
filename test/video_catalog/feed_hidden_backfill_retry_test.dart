import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill_retry.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a hidden feed pauses a scheduled older-page retry', () {
    fakeAsync((clock) {
      final initial = samplePost(id: 'initial');
      final feed = FakeVideoCatalogRepository(forYouFeed: [initial])
        ..olderFeedPages.addAll([
          [initial],
          [initial],
          [initial],
          [samplePost(id: 'fresh')],
        ]);
      final cubit = FeedCubit(
        FeedDependencies(feed: feed, engagement: feed),
        backfillRetry: FeedBackfillRetry(delays: const [Duration(seconds: 1)]),
      );
      unawaited(cubit.load());
      clock.flushMicrotasks();
      expect(feed.olderFeedRequests, hasLength(3));

      cubit.surfaceVisibilityChanged(false);
      clock.elapse(const Duration(seconds: 1));
      clock.flushMicrotasks();
      expect(feed.olderFeedRequests, hasLength(3));

      cubit.surfaceVisibilityChanged(true);
      clock.flushMicrotasks();
      expect(feed.olderFeedRequests, hasLength(4));
      final loaded = cubit.state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), ['initial', 'fresh']);
      unawaited(cubit.close());
      clock.flushMicrotasks();
    });
  });
}
