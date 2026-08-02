import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class VideoFeedPolicy {
  const VideoFeedPolicy();

  List<VideoPost> select({
    required FeedKind kind,
    required List<VideoPost> posts,
    required Set<ProfileId> followed,
    required Set<ProfileId> blocked,
  }) {
    final visible = posts.where(
      (post) => !blocked.contains(post.creator.id),
    );
    final selected = kind == FeedKind.following
        ? visible.where((post) => followed.contains(post.creator.id))
        : visible;
    return List<VideoPost>.unmodifiable(selected);
  }
}
