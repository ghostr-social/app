import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class VideoEngagementRepository {
  Future<VideoPost> toggleLike(VideoPost post);
}
