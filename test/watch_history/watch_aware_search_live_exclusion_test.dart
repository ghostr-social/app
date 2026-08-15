import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_search_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/live_video_search_updates.dart';
import '../support/sample_data.dart';

void main() {
  test('passive search snapshots never expose watched video rows', () async {
    final watched = samplePost(id: 'watched');
    final fresh = samplePost(id: 'fresh');
    final source = FakeVideoCatalogRepository(forYouFeed: const []);
    final updates = LiveVideoSearchUpdates();
    addTearDown(updates.close);
    final search = WatchAwareVideoSearchRepository(
      search: source,
      updates: updates,
      history: FakeWatchHistoryRepository(
        entries: [
          WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 15)),
        ],
      ),
      failureReporter: RecordingFailureReporter(),
    );

    final snapshot = search.watchVideos('clip').first;
    updates.publish('clip', VideoFeedPage(posts: [watched, fresh]));

    expect((await snapshot).page.posts.map((post) => post.id.value), ['fresh']);
  });
}
