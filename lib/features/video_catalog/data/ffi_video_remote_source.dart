import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_post_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/video/video.dart';

typedef FfiVideoInventoryLoader = Future<List<FfiVideoDownload>> Function();
typedef NostrVideoSnapshotLoader = List<VideoPost> Function();

class FfiVideoRemoteSource implements RemoteVideoSource {
  FfiVideoRemoteSource({
    required this.gatewayBaseUrl,
    required NostrVideoSnapshotLoader snapshotLoader,
    FfiVideoInventoryLoader loader = ffiGetDiscoveredVideos,
  })  : _snapshotLoader = snapshotLoader,
        _loader = loader,
        _mapper = FfiVideoPostMapper(gatewayBaseUrl);

  final String gatewayBaseUrl;
  final NostrVideoSnapshotLoader _snapshotLoader;
  final FfiVideoInventoryLoader _loader;
  final FfiVideoPostMapper _mapper;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
  }) async {
    final nativeVideos = await _loadNativeVideos();
    var posts = _mapper.map(nativeVideos, _snapshotLoader());
    if (creatorIds != null) {
      posts = posts.where((post) => creatorIds.contains(post.creator.id));
    }
    if (searchQuery != null) {
      posts = posts.where((post) => _matches(post, searchQuery));
    }
    return posts.toList();
  }

  Future<List<FfiVideoDownload>> _loadNativeVideos() async {
    try {
      return await _loader();
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.video.native',
        message: 'The native video inventory is unavailable.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  bool _matches(VideoPost post, String query) {
    final value = query.trim().toLowerCase();
    return post.caption.toLowerCase().contains(value) ||
        post.songName.toLowerCase().contains(value) ||
        post.creator.displayName.toLowerCase().contains(value) ||
        post.creator.handle.toLowerCase().contains(value);
  }
}
