import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'blocking removes the creator while the replacement watch commits',
    () async {
      final history = _SecondWatchGatedHistory();
      final blocked = samplePost(
        id: 'blocked',
        creator: sampleCreator(id: 'blocked-creator'),
      );
      final replacement = samplePost(id: 'replacement');
      final source = FakeVideoCatalogRepository(
        forYouFeed: [blocked, replacement],
      );
      final cubit = FeedCubit(
        FeedDependencies(
          feed: source,
          engagement: source,
          optional: FeedOptionalDependencies(
            social: source,
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
      );
      addTearDown(() async {
        if (!history.release.isCompleted) history.release.complete();
        await cubit.close();
      });
      await cubit.load();

      final blocking = cubit.blockCreator(blocked);
      await history.secondStarted.future;
      expect((cubit.state as FeedLoaded).posts.single.id.value, 'replacement');

      cubit.commentsPublished(replacement, 1);
      final commented = cubit.state as FeedLoaded;
      expect(commented.posts.single.id.value, 'replacement');
      expect(commented.posts.single.commentCount, replacement.commentCount + 1);

      history.release.complete();
      await blocking;
      expect((cubit.state as FeedLoaded).posts.single.commentCount, 10);
    },
  );
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes += 1;
    if (writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
