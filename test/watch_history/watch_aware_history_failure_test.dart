import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('returns the unfiltered feed and reports when history cannot load',
      () async {
    final reporter = RecordingFailureReporter();
    final repository = WatchAwareVideoFeedRepository(
      feed: FakeVideoCatalogRepository(
        forYouFeed: [samplePost(id: 'one'), samplePost(id: 'two')],
      ),
      history: _FailingWatchHistoryRepository(),
      settings: FakeAppSettingsRepository(AppSettings.defaults()),
      failureReporter: reporter,
    );

    final posts =
        await repository.loadFeed(FeedKind.forYou, excludeWatched: true);

    expect(posts.map((post) => post.id.value), ['one', 'two']);
    expect(reporter.sources, ['WatchAwareVideoFeedRepository.history']);
  });
}

class _FailingWatchHistoryRepository extends FakeWatchHistoryRepository {
  @override
  Future<List<WatchHistoryEntry>> load() async {
    throw const AppFailure('Watch history unavailable.');
  }
}
