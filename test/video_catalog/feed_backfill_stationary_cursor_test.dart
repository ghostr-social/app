import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill_retry.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a stationary older-page cursor cannot start a retry loop', () {
    fakeAsync((clock) {
      final repository = _StationaryCursorFeed(
        forYouFeed: [
          for (var index = 0; index < 12; index += 1)
            samplePost(id: 'post-$index'),
        ],
      );
      final cubit = FeedCubit(
        FeedDependencies(feed: repository, engagement: repository),
        backfillRetry: FeedBackfillRetry(delays: const [Duration(seconds: 1)]),
      );
      cubit.load();
      clock.flushMicrotasks();

      cubit.pageChanged(10);
      clock.flushMicrotasks();
      expect(repository.olderFeedRequests, hasLength(1));

      clock.elapse(const Duration(seconds: 5));
      clock.flushMicrotasks();
      expect(repository.olderFeedRequests, hasLength(1));
      cubit.close();
      clock.flushMicrotasks();
    });
  });
}

final class _StationaryCursorFeed extends FakeVideoCatalogRepository {
  _StationaryCursorFeed({required super.forYouFeed});

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    olderFeedRequests.add(olderThan);
    return VideoFeedPage(posts: const <VideoPost>[], nextOlderThan: olderThan);
  }
}
