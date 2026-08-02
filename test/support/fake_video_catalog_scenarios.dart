import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class FakeFeedScenario {
  const FakeFeedScenario({
    this.followingFeed,
    this.searchResults,
    this.profiles = const {},
    this.failure,
  });

  final List<VideoPost>? followingFeed;
  final List<VideoPost>? searchResults;
  final Map<String, ProfileDetails> profiles;
  final AppFailure? failure;
}

class FakeCommentsScenario {
  const FakeCommentsScenario({
    this.commentsByPost = const {},
    this.failure,
    this.response,
  });

  final Map<String, List<VideoComment>> commentsByPost;
  final AppFailure? failure;
  final Future<List<VideoComment>>? response;
}

class FakeWriteScenario {
  const FakeWriteScenario({this.likeFailure, this.publishFailure});

  final AppFailure? likeFailure;
  final AppFailure? publishFailure;
}
