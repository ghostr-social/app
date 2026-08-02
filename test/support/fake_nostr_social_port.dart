import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class FakeNostrSocialPort implements NostrSocialPort {
  FakeNostrSocialPort({
    Set<ProfileId>? followedProfiles,
    Set<ProfileId>? blockedProfiles,
  })  : followedProfiles = followedProfiles ?? <ProfileId>{},
        blockedProfiles = blockedProfiles ?? <ProfileId>{};

  final Set<ProfileId> followedProfiles;
  final Set<ProfileId> blockedProfiles;
  AppFailure? loadFailure;
  AppFailure? toggleFailure;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    if (loadFailure case final failure?) throw failure;
    return {...blockedProfiles};
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    if (loadFailure case final failure?) throw failure;
    return {...followedProfiles};
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    if (toggleFailure case final failure?) throw failure;
    if (followedProfiles.remove(profileId)) return false;
    followedProfiles.add(profileId);
    return true;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    if (toggleFailure case final failure?) throw failure;
    if (blockedProfiles.remove(profileId)) return false;
    blockedProfiles.add(profileId);
    return true;
  }
}
