import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

/// A feed bound to one search query or `#hashtag`.
///
/// Feed kind and watch exclusion do not apply: the viewer asked for exactly
/// this content, so every match plays — watched or not. The search never
/// reports itself finished either; as long as the viewer keeps swiping, the
/// feed keeps hunting for more matches.
class QueryVideoFeedRepository implements VideoFeedRepository {
  const QueryVideoFeedRepository({
    required VideoSearchRepository search,
    required String query,
  })  : _search = search,
        _query = query;

  final VideoSearchRepository _search;
  final String _query;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final page = await _search.searchVideos(_query);
    return page.posts;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    final page = await _search.loadMoreVideos(_query);
    if (page.hasMore) return page;
    if (page.posts.isNotEmpty) return _continued(page);
    return VideoFeedPage(posts: const <VideoPost>[], nextOlderThan: olderThan);
  }

  // A final page must still leave the feed a way forward.
  VideoFeedPage _continued(VideoFeedPage page) {
    var oldest = page.posts.first.publishedAt;
    for (final post in page.posts.skip(1)) {
      if (post.publishedAt.isBefore(oldest)) oldest = post.publishedAt;
    }
    return VideoFeedPage(
      posts: page.posts,
      nextOlderThan: oldest.subtract(const Duration(seconds: 1)),
    );
  }
}
