import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class PlayableRemoteVideoSource
    implements
        RemoteVideoSource,
        FollowingRemoteVideoSource,
        FollowingRemoteVideoUpdates {
  const PlayableRemoteVideoSource({
    required RemoteVideoSource source,
    required VideoPlaybackCapabilities capabilities,
  }) : _source = source,
       _capabilities = capabilities;

  final RemoteVideoSource _source;
  final VideoPlaybackCapabilities _capabilities;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    final posts = await _source.loadRemoteFeed(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
      olderThan: olderThan,
    );
    return _playable(posts);
  }

  @override
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    final posts = await _source.loadMoreRemoteFeed(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
    );
    return _playable(posts);
  }

  @override
  Future<List<VideoPost>> loadFollowingRemoteFeed(
    FollowingFeedScope scope, {
    DateTime? olderThan,
  }) async {
    final source = _source;
    final posts = source is FollowingRemoteVideoSource
        ? await (source as FollowingRemoteVideoSource).loadFollowingRemoteFeed(
            scope,
            olderThan: olderThan,
          )
        : await source.loadRemoteFeed(
            creatorIds: scope.creators,
            olderThan: olderThan,
          );
    return _playable(posts);
  }

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    return _source
        .watchRemoteFeed(
          creatorIds: creatorIds,
          searchQuery: searchQuery,
          hashtags: hashtags,
        )
        .map(_playableSnapshot);
  }

  @override
  Stream<RemoteVideoSnapshot> watchFollowingRemoteFeed(
    FollowingFeedScope scope,
  ) {
    final source = _source;
    final snapshots = source is FollowingRemoteVideoUpdates
        ? (source as FollowingRemoteVideoUpdates).watchFollowingRemoteFeed(
            scope,
          )
        : source.watchRemoteFeed(creatorIds: scope.creators);
    return snapshots.map(_playableSnapshot);
  }

  List<VideoPost> _playable(List<VideoPost> posts) {
    return posts
        .where((post) => _capabilities.supports(post.media))
        .toList(growable: false);
  }

  RemoteVideoSnapshot _playableSnapshot(RemoteVideoSnapshot snapshot) {
    return RemoteVideoSnapshot(
      revision: snapshot.revision,
      phase: snapshot.phase,
      posts: _playable(snapshot.posts),
    );
  }
}
