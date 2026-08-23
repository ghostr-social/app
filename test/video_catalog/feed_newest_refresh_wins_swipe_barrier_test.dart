import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('newest refresh wins while both wait for the same swipe', () async {
    final first = samplePost(id: 'first');
    final second = samplePost(id: 'second');
    final stale = samplePost(id: 'stale');
    final fresh = samplePost(id: 'fresh');
    final source = _ControlledRefreshSource([first, second]);
    final history = _SecondWatchGatedHistory();
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
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      await cubit.close();
    });
    await cubit.load();

    cubit.pageChanged(1);
    await history.secondStarted.future;
    final older = cubit.refresh();
    await source.firstRead.future;
    source.requests.first.complete(_snapshot([second, stale]));
    await pumpEventQueue();
    final newest = cubit.refresh();
    await source.secondRead.future;
    source.requests.last.complete(_snapshot([second, fresh]));
    await pumpEventQueue();
    history.release.complete();
    await Future.wait([older, newest]);

    final ids = (cubit.state as FeedLoaded).posts.map((post) => post.id.value);
    expect(ids, contains('fresh'));
    expect(ids, isNot(contains('stale')));
  });
}

VideoFeedRefreshSnapshot _snapshot(List<VideoPost> posts) {
  return VideoFeedRefreshSnapshot(allPosts: posts, eligiblePosts: posts);
}

final class _ControlledRefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _ControlledRefreshSource(List<VideoPost> forYouFeed)
    : super(forYouFeed: forYouFeed);

  final requests = <Completer<VideoFeedRefreshSnapshot>>[];
  final firstRead = Completer<void>();
  final secondRead = Completer<void>();

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) {
    final request = Completer<VideoFeedRefreshSnapshot>();
    requests.add(request);
    (requests.length == 1 ? firstRead : secondRead).complete();
    return request.future;
  }
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (++writes != 2) return super.record(entry);
    secondStarted.complete();
    await release.future;
    await super.record(entry);
  }
}
