import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';

/// Names the Rust feed that should stay live for each app feed.
final class RemoteVideoFeedUpdates
    implements VideoFeedUpdates, VideoFeedUpdateRefreshPolicy {
  RemoteVideoFeedUpdates({
    required RemoteVideoUpdates remote,
    required FollowingFeedScopeReader followingScopes,
  }) : _remote = remote,
       _followingScopes = followingScopes;

  final RemoteVideoUpdates _remote;
  final FollowingFeedScopeReader _followingScopes;
  FollowingFeedScope? _activeScope;
  FollowingFeedScope? _preparedScope;
  bool _hasActiveScope = false;

  @override
  Stream<VideoFeedUpdate> watchFeed(FeedKind kind) async* {
    final scope = _preparedScope ?? await _scope(kind);
    _preparedScope = null;
    _activeScope = scope;
    _hasActiveScope = true;
    await for (final snapshot in _watch(kind, scope)) {
      yield VideoFeedUpdate(
        revision: snapshot.revision,
        phase: _phase(snapshot.phase),
        hasPosts: snapshot.posts.isNotEmpty,
      );
    }
  }

  Stream<RemoteVideoSnapshot> _watch(FeedKind kind, FollowingFeedScope? scope) {
    final remote = _remote;
    if (kind == FeedKind.following &&
        scope != null &&
        remote is FollowingRemoteVideoUpdates) {
      return (remote as FollowingRemoteVideoUpdates).watchFollowingRemoteFeed(
        scope,
      );
    }
    return remote.watchRemoteFeed(creatorIds: scope?.creators);
  }

  @override
  Future<bool> shouldRebind(FeedKind kind) async {
    final scope = await _scope(kind);
    if (_hasActiveScope && _sameScope(_activeScope, scope)) {
      return false;
    }
    _preparedScope = scope;
    return true;
  }

  Future<FollowingFeedScope?> _scope(FeedKind kind) {
    if (kind == FeedKind.following) return _followingScopes.load();
    return Future.value();
  }

  bool _sameScope(FollowingFeedScope? left, FollowingFeedScope? right) {
    if (left == null || right == null) return left == right;
    return left.sameAs(right);
  }

  VideoFeedUpdatePhase _phase(RemoteVideoPhase phase) {
    return switch (phase) {
      RemoteVideoPhase.loading => VideoFeedUpdatePhase.loading,
      RemoteVideoPhase.settled => VideoFeedUpdatePhase.settled,
      RemoteVideoPhase.failed => VideoFeedUpdatePhase.failed,
    };
  }
}
