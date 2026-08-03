import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('records the visible post in watch history as pages change', () async {
    final history = FakeWatchHistoryRepository();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'one'), samplePost(id: 'two')],
    );
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      watchTracker: WatchHistoryTracker(
        history: history,
        settings: FakeAppSettingsRepository(AppSettings.defaults()),
        failureReporter: RecordingFailureReporter(),
      ),
    ));
    addTearDown(cubit.close);

    await cubit.load();
    await pumpEventQueue();
    expect(history.entries.map((entry) => entry.videoId), ['e:one']);

    cubit.pageChanged(1);
    await pumpEventQueue();
    expect(history.entries.map((entry) => entry.videoId), ['e:two', 'e:one']);
  });
}
