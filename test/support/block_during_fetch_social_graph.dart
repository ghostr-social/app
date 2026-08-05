import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class BlockDuringFetchSocialGraph implements SocialGraphRepository {
  final Set<ProfileId> _blocked = <ProfileId>{};
  var blockedReads = 0;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    blockedReads += 1;
    return Set<ProfileId>.unmodifiable(_blocked);
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => const <ProfileId>{};

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    return _blocked.add(profileId);
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
