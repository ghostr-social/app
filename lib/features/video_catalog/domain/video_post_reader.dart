import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

abstract interface class VideoPostReader {
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  });

  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  });
}

abstract interface class FollowingVideoPostReader {
  Future<List<VideoPost>> loadFollowing(FollowingFeedScope scope);

  Future<List<VideoPost>> loadOlderFollowing({
    required DateTime olderThan,
    required FollowingFeedScope scope,
  });
}
