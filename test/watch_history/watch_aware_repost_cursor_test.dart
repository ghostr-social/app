import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/repost_samples.dart';
import '../support/sample_data.dart';

void main() {
  test('digs behind a watched repost using its feed activity time', () async {
    final watched = repostedPost();
    final history = FakeWatchHistoryRepository();
    await history.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
    );
    final inner = FakeVideoCatalogRepository(forYouFeed: [watched])
      ..olderFeedPages.add([samplePost(id: 'fresh')]);
    final feed = WatchAwareVideoFeedRepository(
      feed: inner,
      history: history,
      failureReporter: RecordingFailureReporter(),
    );

    await feed.loadFeed(FeedKind.forYou, excludeWatched: true);

    expect(
      inner.olderFeedRequests.single,
      DateTime.utc(2026, 2, 1).subtract(const Duration(seconds: 1)),
    );
  });
}
