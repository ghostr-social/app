import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('refresh keeps every unwatched row eligible', () async {
    final posts = [samplePost(id: 'watched'), samplePost(id: 'fresh')];
    final repository = WatchAwareVideoFeedRepository(
      feed: FakeVideoCatalogRepository(forYouFeed: posts),
      history: FakeWatchHistoryRepository(),
      failureReporter: RecordingFailureReporter(),
    );

    final snapshot = await repository.loadRefresh(FeedKind.forYou);

    expect(snapshot.allPosts, posts);
    expect(snapshot.eligiblePosts, posts);
  });
}
