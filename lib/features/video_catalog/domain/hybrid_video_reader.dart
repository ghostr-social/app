import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_canonicalizer.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

const _feedHydrationBudget = Duration(milliseconds: 100);

class HybridVideoReader implements VideoPostReader, FollowingVideoPostReader {
  const HybridVideoReader({
    required RemoteVideoSource remote,
    required PublishedVideoStore local,
    required NostrVideoInteractions interactions,
    required FailureReporter failureReporter,
  }) : _remote = remote,
       _local = local,
       _interactions = interactions,
       _failureReporter = failureReporter;

  final RemoteVideoSource _remote;
  final PublishedVideoStore _local;
  final NostrVideoInteractions _interactions;
  final FailureReporter _failureReporter;

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    final localPosts = await _loadLocal();
    try {
      final remotePosts = await _remote.loadRemoteFeed(
        creatorIds: creatorIds,
        searchQuery: searchQuery,
        hashtags: hashtags,
      );
      // Hydration failures remain caller-visible rather than remote fallbacks.
      // ignore: unawaited_return_in_try_block
      return _hydrate(_merge(localPosts, remotePosts));
    } on AppFailure catch (error, stackTrace) {
      _report('HybridVideoReader.load', error, stackTrace);
      if (localPosts.isEmpty) rethrow;
      return localPosts;
    }
  }

  @override
  Future<List<VideoPost>> loadFollowing(FollowingFeedScope scope) async {
    final remote = _remote;
    if (remote is! FollowingRemoteVideoSource) {
      return load(creatorIds: scope.creators);
    }
    final localPosts = await _loadLocal();
    final following = remote as FollowingRemoteVideoSource;
    try {
      final remotePosts = await following.loadFollowingRemoteFeed(scope);
      return await _hydrate(_merge(localPosts, remotePosts));
    } on AppFailure catch (error, stackTrace) {
      _report('HybridVideoReader.loadFollowing', error, stackTrace);
      if (localPosts.isEmpty) rethrow;
      return localPosts;
    }
  }

  // Older pages come from relays alone: locally published posts are already
  // part of the first load, and there is no local fallback for the past.
  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) async {
    final posts = await _remote.loadRemoteFeed(
      creatorIds: creatorIds,
      olderThan: olderThan,
    );
    return _hydrate(posts);
  }

  @override
  Future<List<VideoPost>> loadOlderFollowing({
    required DateTime olderThan,
    required FollowingFeedScope scope,
  }) {
    final remote = _remote;
    if (remote is! FollowingRemoteVideoSource) {
      return loadOlder(olderThan: olderThan, creatorIds: scope.creators);
    }
    final following = remote as FollowingRemoteVideoSource;
    return following
        .loadFollowingRemoteFeed(scope, olderThan: olderThan)
        .then(_hydrate);
  }

  Future<List<VideoPost>> _loadLocal() async {
    try {
      return await _local.loadPublishedPosts();
    } on AppFailure catch (error, stackTrace) {
      _report('HybridVideoReader.loadLocal', error, stackTrace);
      return const <VideoPost>[];
    }
  }

  Future<List<VideoPost>> _hydrate(List<VideoPost> posts) {
    return _interactions.hydrateAll(posts, budget: _feedHydrationBudget);
  }

  List<VideoPost> _merge(List<VideoPost> local, List<VideoPost> remote) {
    return canonicalVideoPosts(<VideoPost>[...local, ...remote]);
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: source,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
