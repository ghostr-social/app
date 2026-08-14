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
    final visible = posts.where((post) => !_blocked(post, blocked));
    final selected = kind == FeedKind.following
        ? visible.where((post) => followed.contains(_feedActor(post)))
        : visible;
    return List<VideoPost>.unmodifiable(selected);
  }

  bool _blocked(VideoPost post, Set<ProfileId> blocked) {
    final reposter = post.repost?.reposter.id;
    return blocked.contains(post.creator.id) ||
        reposter != null && blocked.contains(reposter);
  }

  ProfileId _feedActor(VideoPost post) {
    return post.repost?.reposter.id ?? post.creator.id;
  }
}
