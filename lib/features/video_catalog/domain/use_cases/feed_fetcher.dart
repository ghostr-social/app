import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
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
  FeedFetched(
    this.page, {
    List<VideoPost>? eligiblePosts,
    Set<ProfileId> blockedProfiles = const {},
  }) : eligiblePosts = List<VideoPost>.unmodifiable(
         eligiblePosts ?? page.posts,
       ),
       blockedProfiles = Set<ProfileId>.unmodifiable(blockedProfiles);

  final VideoFeedPage page;
  final List<VideoPost> eligiblePosts;
  final Set<ProfileId> blockedProfiles;

  List<VideoPost> get posts => page.posts;
}

/// The relays gave nothing back.
final class FeedUnavailable extends FeedFetch {
  const FeedUnavailable(this.failure);

  final FeedOperationFailure failure;
}

/// Reads the feed and captures every failure as an explicit outcome.
typedef LoadBlockedProfiles = Future<Set<ProfileId>> Function();

final class FeedFetcher {
  const FeedFetcher(this._feed, {LoadBlockedProfiles? loadBlockedProfiles})
    : _loadBlockedProfiles = loadBlockedProfiles;

  final VideoFeedRepository _feed;
  final LoadBlockedProfiles? _loadBlockedProfiles;

  /// The newest posts the viewer has not watched yet.
  Future<FeedFetch> unwatched(FeedKind kind) {
    return _guarded(() => _feed.loadFeed(kind, excludeWatched: true));
  }

  /// The feed as the relays see it now, watched posts included, so the video
  /// the viewer is on can still be found.
  Future<FeedFetch> resync(FeedKind kind) async {
    try {
      final snapshot = await _refresh(kind);
      final blocked = await _blockedProfiles();
      return FeedFetched(
        VideoFeedPage(posts: snapshot.allPosts),
        eligiblePosts: snapshot.eligiblePosts,
        blockedProfiles: blocked,
      );
    } on Object catch (error, stackTrace) {
      return FeedUnavailable(FeedOperationFailure(error, stackTrace));
    }
  }

  Future<VideoFeedRefreshSnapshot> _refresh(FeedKind kind) async {
    if (_feed case final VideoFeedRefreshRepository refresh) {
      return refresh.loadRefresh(kind);
    }
    final posts = await _feed.loadFeed(kind);
    return VideoFeedRefreshSnapshot(allPosts: posts, eligiblePosts: posts);
  }

  Future<Set<ProfileId>> _blockedProfiles() {
    return _loadBlockedProfiles?.call() ?? Future.value(const <ProfileId>{});
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
