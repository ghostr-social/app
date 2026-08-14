import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class VideoFeedRefreshSnapshot {
  VideoFeedRefreshSnapshot({
    required List<VideoPost> allPosts,
    required List<VideoPost> eligiblePosts,
  }) : allPosts = List<VideoPost>.unmodifiable(allPosts),
       eligiblePosts = List<VideoPost>.unmodifiable(eligiblePosts);

  final List<VideoPost> allPosts;
  final List<VideoPost> eligiblePosts;
}

abstract interface class VideoFeedRefreshRepository {
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind);
}
