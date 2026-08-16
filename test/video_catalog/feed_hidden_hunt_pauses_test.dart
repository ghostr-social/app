import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('an empty feed hunts only while its surface is visible', () {
    fakeAsync((clock) {
      final history = FakeWatchHistoryRepository();
      final feed = ScriptedFeedRepository(
        loads: [
          const [],
          [samplePost(id: 'found')],
        ],
      );
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: FakeVideoCatalogRepository(forYouFeed: const []),
          optional: FeedOptionalDependencies(
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
        hunt: FeedHunt(base: const Duration(seconds: 2)),
      );
      unawaited(cubit.load());
      clock.flushMicrotasks();
      expect(cubit.state, isA<FeedEmpty>());

      cubit.surfaceVisibilityChanged(false);
      clock.elapse(const Duration(seconds: 2));
      expect(feed.loadCalls, 1);
      expect(history.entries, isEmpty);

      cubit.surfaceVisibilityChanged(true);
      clock.elapse(const Duration(seconds: 2));
      expect(cubit.state, isA<FeedLoaded>());
      expect(history.entries.single.videoId, 'e:found');
      unawaited(cubit.close());
      clock.flushMicrotasks();
    });
  });
}
