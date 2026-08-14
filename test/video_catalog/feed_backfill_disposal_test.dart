import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('closing the feed stops an in-flight backfill drain', () {
    fakeAsync((clock) {
      final feed = _PendingBackfillFeed();
      final engagement = FakeVideoCatalogRepository(forYouFeed: const []);
      final cubit = FeedCubit(
        FeedDependencies(feed: feed, engagement: engagement),
      );

      cubit.load();
      clock.flushMicrotasks();
      expect(feed.olderRequests, 1);

      cubit.close();
      clock.flushMicrotasks();
      feed.completeFirstDryPage();
      clock.flushMicrotasks();

      expect(feed.olderRequests, 1);
    });
  });
}

final class _PendingBackfillFeed implements VideoFeedRepository {
  final _firstOlder = Completer<VideoFeedPage>();
  late DateTime _cursor;
  int olderRequests = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    return [samplePost(id: 'current')];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    olderRequests += 1;
    _cursor = olderThan.subtract(const Duration(seconds: 1));
    if (olderRequests == 1) return _firstOlder.future;
    return Future.value(VideoFeedPage(posts: const [], nextOlderThan: _cursor));
  }

  void completeFirstDryPage() {
    _firstOlder.complete(
      VideoFeedPage(posts: const [], nextOlderThan: _cursor),
    );
  }
}
