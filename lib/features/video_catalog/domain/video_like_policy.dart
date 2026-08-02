import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class VideoLikePolicy {
  const VideoLikePolicy();

  VideoPost toggle(VideoPost post) {
    final liked = !post.viewerHasLiked;
    return post.withInteraction(
      likeCount: post.likeCount + (liked ? 1 : -1),
      viewerHasLiked: liked,
    );
  }
}
