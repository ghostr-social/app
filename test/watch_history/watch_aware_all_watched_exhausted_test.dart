import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'an exhausted feed comes back empty rather than replaying watched',
    () async {
      final older = samplePost(id: 'older-1');
      final newer = samplePost(id: 'newer-1');
      final history = FakeWatchHistoryRepository();
      await history.record(
        WatchHistoryEntry.fromPost(newer, DateTime.utc(2026, 8, 2)),
      );
      await history.record(
        WatchHistoryEntry.fromPost(older, DateTime.utc(2026, 8, 1)),
      );
      final feed = WatchAwareVideoFeedRepository(
        feed: FakeVideoCatalogRepository(forYouFeed: [newer, older]),
        history: history,
        failureReporter: RecordingFailureReporter(),
      );

      final posts = await feed.loadFeed(FeedKind.forYou, excludeWatched: true);

      expect(posts, isEmpty);
    },
  );
}
