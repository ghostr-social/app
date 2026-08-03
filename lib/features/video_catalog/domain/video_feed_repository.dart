import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class VideoFeedRepository {
  Future<List<VideoPost>> loadFeed(FeedKind kind, {bool excludeWatched});

  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched,
  });
}
