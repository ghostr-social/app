import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('a failed refresh keeps a concurrently published comment', () async {
    final original = samplePost();
    final repository = _GatedFailingRefresh(original);
    final cubit = FeedCubit(
      FeedDependencies(feed: repository, engagement: repository),
    );
    addTearDown(cubit.close);
    await cubit.load();

    final refresh = cubit.refresh();
    await repository.refreshStarted.future;
    cubit.commentsPublished(original, 1);
    repository.releaseRefresh.complete();
    await refresh;

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.single.commentCount, original.commentCount + 1);
    expect(loaded.notice, 'No relay reachable.');
  });
}

final class _GatedFailingRefresh
    implements VideoFeedRepository, VideoEngagementRepository {
  _GatedFailingRefresh(this.original);

  final VideoPost original;
  final refreshStarted = Completer<void>();
  final releaseRefresh = Completer<void>();
  var _loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    _loads += 1;
    if (_loads == 1) return [original];
    refreshStarted.complete();
    await releaseRefresh.future;
    throw const AppFailure('No relay reachable.');
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async => VideoFeedPage(posts: const []);

  @override
  Future<VideoPost> toggleLike(VideoPost post) => throw UnimplementedError();
}
