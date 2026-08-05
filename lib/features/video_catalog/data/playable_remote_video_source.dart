import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

final class PlayableRemoteVideoSource implements RemoteVideoSource {
  const PlayableRemoteVideoSource({
    required RemoteVideoSource source,
    required VideoPlaybackCapabilities capabilities,
  })  : _source = source,
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
