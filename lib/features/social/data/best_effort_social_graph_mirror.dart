import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/social/domain/social_graph_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

final class BestEffortSocialGraphMirror {
  const BestEffortSocialGraphMirror(this._failureReporter);

  final FailureReporter _failureReporter;

  Future<void> saveFollowed(
    SocialGraphStore store,
    Set<ProfileId> profiles,
  ) {
    return _bestEffort(
      'SocialGraphCache.cacheFollow',
      () => store.saveFollowedProfiles(profiles),
    );
  }

  Future<void> saveBlocked(
    SocialGraphStore store,
    Set<ProfileId> profiles,
  ) {
    return _bestEffort(
      'SocialGraphCache.cacheBlock',
      () => store.saveBlockedProfiles(profiles),
    );
  }

  Future<void> applyFollow(
    SocialGraphStore store,
    ProfileId profile,
    bool included,
  ) {
    return _bestEffort(
      'SocialGraphCache.cacheFollow',
      () => _apply(
        store.loadFollowedProfiles,
        store.saveFollowedProfiles,
        profile,
        included,
      ),
    );
  }

  Future<void> applyBlock(
    SocialGraphStore store,
    ProfileId profile,
    bool included,
  ) {
    return _bestEffort(
      'SocialGraphCache.cacheBlock',
      () => _apply(
        store.loadBlockedProfiles,
        store.saveBlockedProfiles,
        profile,
        included,
      ),
    );
  }

  Future<void> _apply(
    Future<Set<ProfileId>> Function() load,
    Future<void> Function(Set<ProfileId>) save,
    ProfileId profile,
    bool included,
  ) async {
    final profiles = await load();
    included ? profiles.add(profile) : profiles.remove(profile);
    await save(profiles);
  }

  Future<void> _bestEffort(
    String source,
    Future<void> Function() operation,
  ) async {
    try {
      await operation();
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: source,
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
