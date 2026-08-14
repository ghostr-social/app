import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class VideoRepostPolicy {
  const VideoRepostPolicy();

  bool supports(VideoPost post) {
    final reference = post.nostrReference;
    return reference != null &&
        (reference.isProtected || reference.signedEvent != null);
  }

  VideoPost toggle(VideoPost post) {
    return post.withRepost(!post.viewerHasReposted);
  }
}
