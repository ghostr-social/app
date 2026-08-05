import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

/// Names the Rust feed that should stay live for each app feed.
final class RemoteVideoFeedUpdates
    implements VideoFeedUpdates, VideoFeedUpdateRefreshPolicy {
  RemoteVideoFeedUpdates({
    required RemoteVideoUpdates remote,
    required SocialGraphRepository social,
  }) : _remote = remote,
       _social = social;

  final RemoteVideoUpdates _remote;
  final SocialGraphRepository _social;
  Set<ProfileId>? _activeCreators;
  Set<ProfileId>? _preparedCreators;
  bool _hasActiveScope = false;

  @override
  Stream<VideoFeedUpdate> watchFeed(FeedKind kind) async* {
    final creators = _preparedCreators ?? await _creators(kind);
    _preparedCreators = null;
    _activeCreators = _copy(creators);
    _hasActiveScope = true;
    await for (final snapshot in _remote.watchRemoteFeed(
      creatorIds: creators,
    )) {
      yield VideoFeedUpdate(
        revision: snapshot.revision,
        phase: _phase(snapshot.phase),
        hasPosts: snapshot.posts.isNotEmpty,
      );
    }
  }

  @override
  Future<bool> shouldRebind(FeedKind kind) async {
    final creators = await _creators(kind);
    if (_hasActiveScope && _sameCreators(_activeCreators, creators)) {
      return false;
    }
    _preparedCreators = _copy(creators);
    return true;
  }

  Future<Set<ProfileId>?> _creators(FeedKind kind) {
    if (kind == FeedKind.following) return _social.loadFollowedProfiles();
    return Future.value();
  }

  Set<ProfileId>? _copy(Set<ProfileId>? creators) {
    return creators == null ? null : Set<ProfileId>.of(creators);
  }

  bool _sameCreators(Set<ProfileId>? left, Set<ProfileId>? right) {
    if (left == null || right == null) return left == right;
    return left.length == right.length && left.containsAll(right);
  }

  VideoFeedUpdatePhase _phase(RemoteVideoPhase phase) {
    return switch (phase) {
      RemoteVideoPhase.loading => VideoFeedUpdatePhase.loading,
      RemoteVideoPhase.settled => VideoFeedUpdatePhase.settled,
      RemoteVideoPhase.failed => VideoFeedUpdatePhase.failed,
    };
  }
}
