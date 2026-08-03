import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class RememberingRemoteVideoSource implements RemoteVideoSource {
  RememberingRemoteVideoSource(this._source, this._snapshot);

  final RemoteVideoSource _source;
  final NostrVideoSnapshot _snapshot;
  var _fullLoadGeneration = 0;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
  }) async {
    final isFullLoad = creatorIds == null && searchQuery == null;
    final generation = isFullLoad ? ++_fullLoadGeneration : null;
    final posts = await _source.loadRemoteFeed(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
    );
    if (generation != null && generation == _fullLoadGeneration) {
      _snapshot.remember(posts);
    }
    return posts;
  }
}
