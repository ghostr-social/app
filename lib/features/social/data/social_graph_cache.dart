import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class SocialGraphCache implements SocialGraphRepository {
  const SocialGraphCache(this._remote, this._local, this._failureReporter);

  final NostrSocialPort _remote;
  final SocialGraphStore _local;
  final FailureReporter _failureReporter;

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async {
    try {
      final followed = await _remote.loadFollowedProfiles();
      await _local.saveFollowedProfiles(followed);
      return followed;
    } on AppFailure catch (error, stackTrace) {
      _report('SocialGraphCache.loadFollowedProfiles', error, stackTrace);
      return _local.loadFollowedProfiles();
    }
  }

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async {
    try {
      final blocked = await _remote.loadBlockedProfiles();
      await _local.saveBlockedProfiles(blocked);
      return blocked;
    } on AppFailure catch (error, stackTrace) {
      _report('SocialGraphCache.loadBlockedProfiles', error, stackTrace);
      return _local.loadBlockedProfiles();
    }
  }

  @override
  Future<bool> toggleFollow(ProfileId profileId) async {
    final isFollowing = await _remote.toggleFollow(profileId);
    await _cacheFollow(profileId, isFollowing);
    return isFollowing;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async {
    final isBlocked = await _remote.toggleBlock(profileId);
    await _cacheBlock(profileId, isBlocked);
    return isBlocked;
  }

  Future<void> _cacheFollow(ProfileId profileId, bool isFollowing) async {
    final followed = await _local.loadFollowedProfiles();
    isFollowing ? followed.add(profileId) : followed.remove(profileId);
    await _local.saveFollowedProfiles(followed);
  }

  Future<void> _cacheBlock(ProfileId profileId, bool isBlocked) async {
    final blocked = await _local.loadBlockedProfiles();
    isBlocked ? blocked.add(profileId) : blocked.remove(profileId);
    await _local.saveBlockedProfiles(blocked);
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: source,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
