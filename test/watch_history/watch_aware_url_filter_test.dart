import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a republished clip with a watched URL never plays again', () async {
    final watched = samplePost(id: 'original');
    final history = FakeWatchHistoryRepository();
    await history.record(
      WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
    );
    final republished = samplePost(id: 'republished').withMedia(
      VideoMediaSource.remote('https://example.com/video/original.mp4'),
    );
    final fresh = samplePost(id: 'fresh');
    final feed = WatchAwareVideoFeedRepository(
      feed: FakeVideoCatalogRepository(forYouFeed: [republished, fresh]),
      history: history,
      settings: FakeAppSettingsRepository(AppSettings.defaults()),
      failureReporter: RecordingFailureReporter(),
    );

    final posts = await feed.loadFeed(FeedKind.forYou, excludeWatched: true);

    expect(posts.map((post) => post.id.value), ['fresh']);
  });
}
