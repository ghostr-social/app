import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_operation_failure.dart';

/// What one feed request came back with.
sealed class FeedFetch {
  const FeedFetch();
}

/// A slice of the feed, with the cursor for the slice after it.
final class FeedFetched extends FeedFetch {
  FeedFetched(this.page, {List<VideoPost>? eligiblePosts})
    : eligiblePosts = List<VideoPost>.unmodifiable(eligiblePosts ?? page.posts);

  final VideoFeedPage page;
  final List<VideoPost> eligiblePosts;

  List<VideoPost> get posts => page.posts;
}

/// The relays gave nothing back.
final class FeedUnavailable extends FeedFetch {
  const FeedUnavailable(this.failure);

  final FeedOperationFailure failure;
}

/// Reads the feed and captures every failure as an explicit outcome.
final class FeedFetcher {
  const FeedFetcher(this._feed);

  final VideoFeedRepository _feed;

  /// The newest posts the viewer has not watched yet.
  Future<FeedFetch> unwatched(FeedKind kind) {
    return _guarded(() => _feed.loadFeed(kind, excludeWatched: true));
  }

  /// The feed as the relays see it now, watched posts included, so the video
  /// the viewer is on can still be found.
  Future<FeedFetch> resync(FeedKind kind) async {
    try {
      if (_feed case final VideoFeedRefreshRepository refresh) {
        final snapshot = await refresh.loadRefresh(kind);
        return FeedFetched(
          VideoFeedPage(posts: snapshot.allPosts),
          eligiblePosts: snapshot.eligiblePosts,
        );
      }
      return FeedFetched(VideoFeedPage(posts: await _feed.loadFeed(kind)));
    } on Object catch (error, stackTrace) {
      return FeedUnavailable(FeedOperationFailure(error, stackTrace));
    }
  }

  /// One page further into the past.
  Future<FeedFetch> older(FeedKind kind, DateTime olderThan) async {
    try {
      return FeedFetched(
        await _feed.loadOlderFeed(
          kind,
          olderThan: olderThan,
          excludeWatched: true,
        ),
      );
    } on Object catch (error, stackTrace) {
      return FeedUnavailable(FeedOperationFailure(error, stackTrace));
    }
  }

  Future<FeedFetch> _guarded(Future<List<VideoPost>> Function() request) async {
    try {
      return FeedFetched(VideoFeedPage(posts: await request()));
    } on Object catch (error, stackTrace) {
      return FeedUnavailable(FeedOperationFailure(error, stackTrace));
    }
  }
}
