import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a fully watched first page digs deeper instead of replaying',
      () async {
    final watched = samplePost(id: 'watched-1');
    final fresh = samplePost(id: 'fresh-1');
    final history = FakeWatchHistoryRepository();
    await history.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
    );
    final inner = FakeVideoCatalogRepository(forYouFeed: [watched])
      ..olderFeedPages.add([fresh]);
    final feed = WatchAwareVideoFeedRepository(
      feed: inner,
      history: history,
      settings: FakeAppSettingsRepository(AppSettings.defaults()),
      failureReporter: RecordingFailureReporter(),
    );

    final posts = await feed.loadFeed(FeedKind.forYou, excludeWatched: true);

    expect(posts.map((post) => post.id.value), ['fresh-1']);
    expect(inner.olderFeedRequests, hasLength(1));
  });
}
