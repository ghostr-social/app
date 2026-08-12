import 'dart:async';

import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

final class ProfileAggregationProbe
    implements VideoPostReader, SocialGraphRepository {
  final posts = Completer<List<VideoPost>>();
  final followed = Completer<Set<ProfileId>>();
  final blocked = Completer<Set<ProfileId>>();
  final _acceptedFollows = <ProfileId>{};
  var postReads = 0;
  var followedReads = 0;
  var blockedReads = 0;

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    postReads += 1;
    return posts.future;
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() {
    followedReads += 1;
    return followed.future;
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    blockedReads += 1;
    return blocked.future;
  }

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    return _acceptedFollows.add(profileId)
        ? FollowOutcome.newlyFollowed
        : FollowOutcome.alreadyFollowing;
  }

  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) async => const <VideoPost>[];

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    if (_acceptedFollows.remove(profileId)) return false;
    _acceptedFollows.add(profileId);
    return true;
  }

  void release({List<VideoPost> loadedPosts = const <VideoPost>[]}) {
    if (!posts.isCompleted) posts.complete(loadedPosts);
    if (!followed.isCompleted) followed.complete(const <ProfileId>{});
    if (!blocked.isCompleted) blocked.complete(const <ProfileId>{});
  }
}
