import 'package:ghostr/features/video_catalog/domain/video_post.dart';

enum VideoPublicationCacheStatus { stored, unavailable }

final class VideoPublication {
  const VideoPublication({
    required this.post,
    required this.cacheStatus,
  });

  const VideoPublication.stored(this.post)
      : cacheStatus = VideoPublicationCacheStatus.stored;

  final VideoPost post;
  final VideoPublicationCacheStatus cacheStatus;
}
