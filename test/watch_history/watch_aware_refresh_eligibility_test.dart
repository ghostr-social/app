import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'refresh distinguishes full reconciliation rows from eligible rows',
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
        settings: FakeAppSettingsRepository(AppSettings.defaults()),
        failureReporter: RecordingFailureReporter(),
      );

      final result = await FeedFetcher(repository).resync(FeedKind.forYou);
      final fetched = result as FeedFetched;

      expect(fetched.posts.map((post) => post.id.value), ['watched', 'fresh']);
      expect(fetched.eligiblePosts.map((post) => post.id.value), ['fresh']);
    },
  );
}
