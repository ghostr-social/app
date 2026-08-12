import 'package:ghostr/core/async/parallel_wait.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details_policy.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';
import 'package:ghostr/features/video_catalog/domain/video_profile_repository.dart';

class AggregatingVideoProfileRepository implements VideoProfileRepository {
  const AggregatingVideoProfileRepository(
    this._reader,
    this._social, {
    ProfileDetailsPolicy policy = const ProfileDetailsPolicy(),
  }) : _policy = policy;

  final VideoPostReader _reader;
  final SocialGraphRepository _social;
  final ProfileDetailsPolicy _policy;

  @override
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  ) async {
    final postsLoad = _reader.load(creatorIds: <ProfileId>{profileId});
    final followedLoad = _social.loadFollowedProfiles();
    final isCurrentUser = viewer.id == profileId;
    final blockedLoad = isCurrentUser
        ? Future<Set<ProfileId>>.value(<ProfileId>{})
        : _social.loadBlockedProfiles();
    final (posts, social) = await waitForBoth(
      postsLoad,
      waitForBoth(followedLoad, blockedLoad),
    );
    final context = ProfileSocialContext(
      viewer: viewer,
      targetId: profileId,
      followed: social.$1,
      blocked: social.$2,
    );
    return _policy.build(context, posts);
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) {
    return _social.toggleFollow(profileId);
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) {
    return _social.toggleBlock(profileId);
  }
}
