import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

typedef RemoteVideoSourceBuilder = Future<RemoteVideoSource> Function();

class DeferredRemoteVideoSource implements RemoteVideoSource {
  DeferredRemoteVideoSource(this._builder);

  final RemoteVideoSourceBuilder _builder;
  Future<RemoteVideoSource>? _source;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    final source = await (_source ??= _builder());
    return source.loadRemoteFeed(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
    );
  }
}
