import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

typedef FollowingFeedViewer = NostrPublicKeyHex? Function();

final class FollowingFeedScope {
  FollowingFeedScope({required this.viewer, required Set<ProfileId> creators})
    : creators = Set<ProfileId>.unmodifiable(creators);

  final NostrPublicKeyHex viewer;
  final Set<ProfileId> creators;

  bool sameAs(FollowingFeedScope other) {
    return viewer == other.viewer &&
        creators.length == other.creators.length &&
        creators.containsAll(other.creators);
  }
}

final class FollowingFeedScopeReader {
  const FollowingFeedScopeReader(this._social, this._viewer);

  final SocialGraphRepository _social;
  final FollowingFeedViewer _viewer;

  Future<FollowingFeedScope> load() async {
    final viewer = _viewer();
    if (viewer == null) throw const AppFailure('Sign in first.');
    final creators = await _social.loadFollowedProfiles();
    if (_viewer() != viewer) {
      throw const AppFailure('The active account changed. Try again.');
    }
    return FollowingFeedScope(viewer: viewer, creators: creators);
  }
}
