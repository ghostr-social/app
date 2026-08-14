import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('refresh keeps every row when watched filtering is disabled', () async {
    final posts = [samplePost(id: 'watched'), samplePost(id: 'fresh')];
    final repository = WatchAwareVideoFeedRepository(
      feed: FakeVideoCatalogRepository(forYouFeed: posts),
      history: FakeWatchHistoryRepository(),
      settings: FakeAppSettingsRepository(
        AppSettings.defaults().copyWith(hideWatchedVideos: false),
      ),
      failureReporter: RecordingFailureReporter(),
    );

    final snapshot = await repository.loadRefresh(FeedKind.forYou);

    expect(snapshot.allPosts, posts);
    expect(snapshot.eligiblePosts, posts);
  });
}
