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

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
  }) async {
    loadCount += 1;
    requestedCreatorIds = creatorIds;
    requestedSearchQuery = searchQuery;
    if (failure case final failure?) throw failure;
    return posts;
  }
}
