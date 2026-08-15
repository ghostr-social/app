import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('page digging stops after three rounds and keeps the cursor', () async {
    final watched = samplePost(id: 'watched-1');
    final history = FakeWatchHistoryRepository();
    await history.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
    );
    final inner = FakeVideoCatalogRepository(forYouFeed: const []);
    for (var page = 0; page < 4; page += 1) {
      inner.olderFeedPages.add([watched]);
    }
    final feed = WatchAwareVideoFeedRepository(
      feed: inner,
      history: history,
      failureReporter: RecordingFailureReporter(),
    );

    final page = await feed.loadOlderFeed(
      FeedKind.forYou,
      olderThan: DateTime.utc(2026, 8, 2),
      excludeWatched: true,
    );

    expect(page.posts, isEmpty);
    expect(page.hasMore, isTrue);
    expect(inner.olderFeedRequests, hasLength(3));
  });
}
