import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class FakeSocialGraphRepository implements SocialGraphRepository {
  FakeSocialGraphRepository({
    Set<ProfileId>? blocked,
    Set<ProfileId>? followed,
  })  : blocked = blocked ?? <ProfileId>{},
        followed = followed ?? <ProfileId>{};

  final Set<ProfileId> blocked;
  final Set<ProfileId> followed;
  final toggledBlocks = <ProfileId>[];
  AppFailure? loadFailure;
  Object? toggleFailure;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    if (loadFailure case final failure?) throw failure;
    return {...blocked};
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    if (loadFailure case final failure?) throw failure;
    return {...followed};
  }

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    return followed.add(profileId)
        ? FollowOutcome.newlyFollowed
        : FollowOutcome.alreadyFollowing;
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    if (followed.remove(profileId)) return false;
    followed.add(profileId);
    return true;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    if (toggleFailure case final failure?) throw failure;
    toggledBlocks.add(profileId);
    if (blocked.remove(profileId)) return false;
    blocked.add(profileId);
    return true;
  }
}
