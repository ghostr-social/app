import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

class _BrokenSettingsRepository implements AppSettingsRepository {
  @override
  Future<AppSettings> load() async {
    throw const AppFailure('Could not read app settings.');
  }

  @override
  Future<void> save(AppSettings settings) async {}
}

void main() {
  test('keeps the feed unfiltered when the settings read fails', () async {
    final watched = samplePost(id: 'watched-1');
    final fresh = samplePost(id: 'fresh-1');
    final reporter = RecordingFailureReporter();
    final history = FakeWatchHistoryRepository();
    await history
        .record(WatchHistoryEntry.fromPost(watched, DateTime.utc(2026)));
    final feed = WatchAwareVideoFeedRepository(
      feed: FakeVideoCatalogRepository(forYouFeed: [watched, fresh]),
      history: history,
      settings: _BrokenSettingsRepository(),
      failureReporter: reporter,
    );

    final posts = await feed.loadFeed(FeedKind.forYou, excludeWatched: true);

    expect(posts.map((post) => post.id.value), ['watched-1', 'fresh-1']);
    expect(reporter.sources, ['WatchAwareVideoFeedRepository.settings']);
  });
}
