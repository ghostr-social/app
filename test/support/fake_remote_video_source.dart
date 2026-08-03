import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class FakeRemoteVideoSource implements RemoteVideoSource {
  FakeRemoteVideoSource(this.posts);

  final List<VideoPost> posts;
  List<VideoPost> olderPosts = const <VideoPost>[];
  AppFailure? failure;
  int loadCount = 0;
  Set<ProfileId>? requestedCreatorIds;
  String? requestedSearchQuery;
  Set<String>? requestedHashtags;
  final List<DateTime> requestedOlderThan = <DateTime>[];

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    loadCount += 1;
    requestedCreatorIds = creatorIds;
    requestedSearchQuery = searchQuery;
    requestedHashtags = hashtags;
    if (olderThan != null) requestedOlderThan.add(olderThan);
    if (failure case final failure?) throw failure;
    return olderThan == null ? posts : olderPosts;
  }
}
