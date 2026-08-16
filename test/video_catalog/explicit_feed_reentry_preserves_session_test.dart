import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'temporary explicit-feed invisibility preserves the current session',
    () async {
      final history = FakeWatchHistoryRepository();
      final post = samplePost(id: 'profile-video');
      final source = FakeVideoCatalogRepository(forYouFeed: [post]);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: source,
          engagement: source,
          optional: FeedOptionalDependencies(
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
      );
      addTearDown(cubit.close);
      await cubit.load();
      expect(history.entries, hasLength(1));

      cubit.surfaceVisibilityChanged(false);
      expect(cubit.state, isA<FeedLoaded>());
      cubit.surfaceVisibilityChanged(true);
      await pumpEventQueue();

      expect(cubit.state, isA<FeedLoaded>());
      expect(history.entries, hasLength(1));
      expect(history.entries.single.videoId, 'e:profile-video');
    },
  );
}
