import 'package:ghostr/features/video_catalog/domain/video_post.dart';

enum VideoRepostHydration { prompt, patient }

abstract interface class VideoRepostRepository {
  Future<List<VideoPost>> hydrateAll(
    List<VideoPost> posts, {
    VideoRepostHydration mode = VideoRepostHydration.prompt,
  });

  Future<VideoPost> toggleRepost(VideoPost post);
}
