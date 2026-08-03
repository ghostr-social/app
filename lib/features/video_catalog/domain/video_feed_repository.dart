import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class VideoFeedRepository {
  Future<List<VideoPost>> loadFeed(FeedKind kind, {bool excludeWatched});
}
