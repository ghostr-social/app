import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('refresh commits the page already reached by a pending swipe', (
    tester,
  ) async {
    final history = _SecondWatchGatedHistory();
    final first = samplePost(id: 'first', caption: 'First video');
    final second = samplePost(id: 'second', caption: 'Second video');
    final source = _ObservedRefreshSource([first, second]);
    addTearDown(() {
      if (!history.release.isCompleted) history.release.complete();
    });
    await tester.pumpWidget(
      feedScreenHarness(
        source,
        options: FeedScreenHarnessOptions(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();

    await tester.drag(find.byType(Scrollable), const Offset(0, -600));
    await history.secondStarted.future;
    source.forYouFeed.remove(first);
    final refresh = cubit.refresh();
    await source.refreshRead.future;
    await tester.pump();
    cubit.commentsPublished(second, 1);

    history.release.complete();
    await refresh;
    await tester.pumpAndSettle();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.roster.active.id, second.id);
    expect(loaded.roster.active.commentCount, second.commentCount + 1);
    expect(find.text('Second video').hitTestable(), findsOneWidget);
  });
}

final class _ObservedRefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _ObservedRefreshSource(List<VideoPost> posts) : super(forYouFeed: posts);

  final refreshRead = Completer<void>();

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    refreshRead.complete();
    return VideoFeedRefreshSnapshot(
      allPosts: forYouFeed,
      eligiblePosts: forYouFeed,
    );
  }
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (++writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
