import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class VideoLikePolicy {
  const VideoLikePolicy();

  VideoPost toggle(VideoPost post) {
    final liked = !post.viewerHasLiked;
    final count = post.likeCount + (liked ? 1 : -1);
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount: count < 0 ? 0 : count,
        viewerHasLiked: liked,
      ),
    );
  }
}
