import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'refresh appends unseen eligible rows without replaying watched rows',
    () async {
      final watched = samplePost(id: 'watched');
      final current = samplePost(id: 'current');
      final fresh = samplePost(id: 'fresh');
      final inner = FakeVideoCatalogRepository(forYouFeed: [watched, current]);
      final feed = WatchAwareVideoFeedRepository(
        feed: inner,
        history: FakeWatchHistoryRepository(
          entries: [
            WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 3, 12)),
          ],
        ),
        settings: FakeAppSettingsRepository(AppSettings.defaults()),
        failureReporter: RecordingFailureReporter(),
      );
      final cubit = FeedCubit(FeedDependencies(feed: feed, engagement: inner));
      addTearDown(cubit.close);
      await cubit.load();
      inner.forYouFeed.insert(0, fresh);

      await cubit.refresh();

      final loaded = cubit.state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), ['current', 'fresh']);
      expect(loaded.posts[loaded.activeIndex].id.value, 'current');
    },
  );
}
