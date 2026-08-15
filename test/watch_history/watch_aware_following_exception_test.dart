import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('Following remains an explicit watched-video replay surface', () async {
    final watched = samplePost(id: 'watched');
    final repository = WatchAwareVideoFeedRepository(
      feed: FakeVideoCatalogRepository(
        forYouFeed: const [],
        feed: FakeFeedScenario(followingFeed: [watched]),
      ),
      history: FakeWatchHistoryRepository(
        entries: [
          WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
        ],
      ),
      failureReporter: RecordingFailureReporter(),
    );

    final posts = await repository.loadFeed(
      FeedKind.following,
      excludeWatched: true,
    );

    expect(posts, [watched]);
    final refresh = await repository.loadRefresh(FeedKind.following);
    expect(refresh.eligiblePosts, [watched]);
  });
}
