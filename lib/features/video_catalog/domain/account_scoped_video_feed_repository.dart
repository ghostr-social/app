import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class AccountScopedVideoFeedRepository
    implements
        VideoFeedRepository,
        VideoFeedRefreshRepository,
        RepostHydrationStatus {
  const AccountScopedVideoFeedRepository(this._feed, this._viewer);

  final VideoFeedRepository _feed;
  final FollowingFeedViewer _viewer;

  @override
  bool get isRepostHydrated {
    final feed = _feed;
    if (feed case final RepostHydrationStatus status) {
      return status.isRepostHydrated;
    }
    return false;
  }

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    return _guard(() => _feed.loadFeed(kind, excludeWatched: excludeWatched));
  }

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) {
    return _guard(() async {
      final feed = _feed;
      if (feed case final VideoFeedRefreshRepository refresh) {
        return refresh.loadRefresh(kind);
      }
      final posts = await feed.loadFeed(kind, excludeWatched: true);
      return VideoFeedRefreshSnapshot(allPosts: posts, eligiblePosts: posts);
    });
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    return _guard(
      () => _feed.loadOlderFeed(
        kind,
        olderThan: olderThan,
        excludeWatched: excludeWatched,
      ),
    );
  }

  Future<T> _guard<T>(Future<T> Function() operation) async {
    final viewer = _viewer();
    final result = await operation();
    if (_viewer() != viewer) {
      throw const AppFailure('The active account changed. Try again.');
    }
    return result;
  }
}
