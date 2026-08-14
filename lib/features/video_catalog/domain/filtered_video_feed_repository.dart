import 'package:ghostr/core/async/parallel_wait.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

class FilteredVideoFeedRepository implements VideoFeedRepository {
  const FilteredVideoFeedRepository(
    this._reader,
    this._social, {
    VideoFeedPolicy policy = const VideoFeedPolicy(),
    FollowingFeedScopeReader? followingScopes,
  }) : _policy = policy,
       _followingScopes = followingScopes;

  final VideoPostReader _reader;
  final SocialGraphRepository _social;
  final VideoFeedPolicy _policy;
  final FollowingFeedScopeReader? _followingScopes;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final scope = await _scopeFor(kind);
    final followed = scope?.creators ?? const <ProfileId>{};
    final posts = _load(kind, scope);
    final blocked = _social.loadBlockedProfiles();
    final (loadedPosts, _) = await waitForBoth(posts, blocked);
    final selected = await _selectWithFreshBlocks(kind, loadedPosts, followed);
    return selected;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    final scope = await _scopeFor(kind);
    final followed = scope?.creators ?? const <ProfileId>{};
    final posts = _loadOlder(kind, scope, olderThan);
    final blocked = _social.loadBlockedProfiles();
    final (loadedPosts, _) = await waitForBoth(posts, blocked);
    final selected = await _selectWithFreshBlocks(kind, loadedPosts, followed);
    // The cursor advances by what was fetched, not what survived filtering,
    // so pages full of blocked creators cannot stall pagination.
    return VideoFeedPage(
      posts: selected,
      nextOlderThan: _nextCursor(loadedPosts),
    );
  }

  DateTime? _nextCursor(List<VideoPost> fetched) {
    if (fetched.isEmpty) return null;
    var oldest = fetched.first.feedActivityAt;
    for (final post in fetched.skip(1)) {
      if (post.feedActivityAt.isBefore(oldest)) oldest = post.feedActivityAt;
    }
    return oldest.subtract(const Duration(seconds: 1));
  }

  Future<FollowingFeedScope?> _scopeFor(FeedKind kind) async {
    if (kind != FeedKind.following) return null;
    final scopes = _followingScopes;
    if (scopes == null) {
      throw const AppFailure('Following feed scope is unavailable.');
    }
    return scopes.load();
  }

  Future<List<VideoPost>> _load(FeedKind kind, FollowingFeedScope? scope) {
    final reader = _reader;
    if (reader case final FollowingVideoPostReader following
        when scope != null) {
      return following.loadFollowing(scope);
    }
    return reader.load(creatorIds: scope?.creators);
  }

  Future<List<VideoPost>> _loadOlder(
    FeedKind kind,
    FollowingFeedScope? scope,
    DateTime olderThan,
  ) {
    final reader = _reader;
    if (reader case final FollowingVideoPostReader following
        when scope != null) {
      return following.loadOlderFollowing(olderThan: olderThan, scope: scope);
    }
    return reader.loadOlder(olderThan: olderThan, creatorIds: scope?.creators);
  }

  Future<List<VideoPost>> _selectWithFreshBlocks(
    FeedKind kind,
    List<VideoPost> posts,
    Set<ProfileId> followed,
  ) async {
    return _policy.select(
      kind: kind,
      posts: posts,
      followed: followed,
      blocked: await _social.loadBlockedProfiles(),
    );
  }
}
