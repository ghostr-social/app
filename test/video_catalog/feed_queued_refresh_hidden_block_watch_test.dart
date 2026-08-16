import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a hidden block replacement is watched before queued refresh', () async {
    final blocked = samplePost(
      id: 'blocked',
      creator: sampleCreator(id: 'blocked-creator'),
    );
    final replacement = samplePost(id: 'replacement');
    final source = _GatedRefreshSource([blocked, replacement]);
    final social = _GatedSocialGraph();
    final history = FakeWatchHistoryRepository();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          social: social,
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
      if (!source.release.isCompleted) source.release.complete();
      if (!social.release.isCompleted) social.release.complete();
      await cubit.close();
    });
    await cubit.load();

    final blocking = cubit.blockCreator(blocked);
    await social.started.future;
    cubit.surfaceVisibilityChanged(false);
    await cubit.refresh();
    social.release.complete();
    await blocking;
    cubit.surfaceVisibilityChanged(true);
    await source.started.future;
    await pumpEventQueue();

    expect(history.entries.map((entry) => entry.videoId), [
      'e:replacement',
      'e:blocked',
    ]);
  });
}

final class _GatedRefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _GatedRefreshSource(List<VideoPost> posts) : super(forYouFeed: posts);

  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    started.complete();
    await release.future;
    return VideoFeedRefreshSnapshot(
      allPosts: forYouFeed,
      eligiblePosts: forYouFeed,
    );
  }
}

final class _GatedSocialGraph extends FakeSocialGraphRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    started.complete();
    await release.future;
    return super.toggleBlock(profileId);
  }
}
