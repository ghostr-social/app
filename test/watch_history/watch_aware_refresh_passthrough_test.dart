import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'For You filters watched posts even when a caller forgets the flag',
    () async {
      final watched = samplePost(id: 'watched');
      final fresh = samplePost(id: 'fresh');
      final repository = WatchAwareVideoFeedRepository(
        feed: FakeVideoCatalogRepository(forYouFeed: [watched, fresh]),
        history: FakeWatchHistoryRepository(
          entries: [
            WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 3, 12)),
          ],
        ),
        failureReporter: RecordingFailureReporter(),
      );

      final posts = await repository.loadFeed(FeedKind.forYou);

      expect(posts.map((post) => post.id.value), ['fresh']);
    },
  );
}
