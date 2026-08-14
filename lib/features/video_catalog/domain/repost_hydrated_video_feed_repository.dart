import 'package:ghostr/features/reposts/domain/video_repost_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

VideoFeedRepository ensureRepostHydratedVideoFeed(
  VideoFeedRepository feed,
  VideoRepostRepository reposts,
) {
  if (feed case RepostHydrationStatus(isRepostHydrated: true)) return feed;
  return RepostHydratedVideoFeedRepository(feed, reposts);
}

abstract interface class RepostHydrationStatus {
  bool get isRepostHydrated;
}

final class RepostHydratedVideoFeedRepository
    implements VideoFeedRepository, RepostHydrationStatus {
  const RepostHydratedVideoFeedRepository(this._feed, this._reposts);

  final VideoFeedRepository _feed;
  final VideoRepostRepository _reposts;

  @override
  bool get isRepostHydrated => true;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final posts = await _feed.loadFeed(kind, excludeWatched: excludeWatched);
    return _reposts.hydrateAll(posts);
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    final page = await _feed.loadOlderFeed(
      kind,
      olderThan: olderThan,
      excludeWatched: excludeWatched,
    );
    return VideoFeedPage(
      posts: await _reposts.hydrateAll(page.posts),
      nextOlderThan: page.nextOlderThan,
    );
  }
}
