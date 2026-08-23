import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'refresh preserves inflight backfill and fresh media authority',
    () async {
      final initial = [
        for (var index = 0; index < 12; index += 1)
          samplePost(id: 'post-$index'),
      ];
      final fresh = _revision(secondTestEventId, 'fresh.mp4');
      final stale = _revision(testEventId, 'stale.mp4');
      final source = _RefreshSource(initial, [...initial.skip(1), fresh])
        ..olderFeedPages.add([samplePost(id: 'older'), stale]);
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

      final refresh = cubit.refresh();
      await history.secondStarted.future;
      await cubit.loadMore();
      history.release.complete();
      await refresh;

      final loaded = cubit.state as FeedLoaded;
      expect(loaded.roster.active.id.value, 'post-1');
      expect(loaded.posts.any((post) => post.id.value == 'older'), isTrue);
      final revision = loaded.posts.singleWhere(
        (post) => post.nostrReference != null,
      );
      expect(revision.media.remoteUrl, endsWith('/fresh.mp4'));
    },
  );
}

VideoPost _revision(String id, String name) => samplePost(
  id: id,
  nostrReference: nostrReference(
    eventId: id,
    kind: 34236,
    identifier: 'stable-video',
  ),
).withMedia(VideoMediaSource.remote('https://example.com/$name'));

final class _RefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _RefreshSource(List<VideoPost> initial, this.refreshed)
    : super(forYouFeed: initial);

  final List<VideoPost> refreshed;

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async =>
      VideoFeedRefreshSnapshot(allPosts: refreshed, eligiblePosts: refreshed);
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
