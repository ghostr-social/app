import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('replays the least recently watched videos when all are watched',
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
      settings: FakeAppSettingsRepository(AppSettings.defaults()),
      failureReporter: RecordingFailureReporter(),
    );

    final posts = await feed.loadFeed(FeedKind.forYou, excludeWatched: true);

    expect(posts.map((post) => post.id.value), ['older-1', 'newer-1']);
  });
}
