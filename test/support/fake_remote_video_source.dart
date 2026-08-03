import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class FakeRemoteVideoSource implements RemoteVideoSource {
  FakeRemoteVideoSource(this.posts);

  final List<VideoPost> posts;
  AppFailure? failure;
  int loadCount = 0;
  Set<ProfileId>? requestedCreatorIds;
  String? requestedSearchQuery;
  Set<String>? requestedHashtags;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    loadCount += 1;
    requestedCreatorIds = creatorIds;
    requestedSearchQuery = searchQuery;
    requestedHashtags = hashtags;
    if (failure case final failure?) throw failure;
    return posts;
  }
}
