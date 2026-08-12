import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class BlockDuringFetchSocialGraph implements SocialGraphRepository {
  final Set<ProfileId> _blocked = <ProfileId>{};
  final Set<ProfileId> _followed = <ProfileId>{};
  var blockedReads = 0;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    blockedReads += 1;
    return Set<ProfileId>.unmodifiable(_blocked);
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => {..._followed};

  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    return _followed.add(profileId)
        ? FollowOutcome.newlyFollowed
        : FollowOutcome.alreadyFollowing;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    return _blocked.add(profileId);
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    if (_followed.remove(profileId)) return false;
    _followed.add(profileId);
    return true;
  }
}
