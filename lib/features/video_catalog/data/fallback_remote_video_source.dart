import 'dart:developer';

import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

class FallbackRemoteVideoSource implements RemoteVideoSource {
  const FallbackRemoteVideoSource({
    required RemoteVideoSource primary,
    required RemoteVideoSource fallback,
  })  : _primary = primary,
        _fallback = fallback;

  final RemoteVideoSource _primary;
  final RemoteVideoSource _fallback;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    try {
      final posts = await _primary.loadRemoteFeed(
        creatorIds: creatorIds,
        searchQuery: searchQuery,
        hashtags: hashtags,
      );
      if (posts.isNotEmpty) return posts;
    } on Object catch (error, stackTrace) {
      log(
        'Primary video source failed; using the warmed fallback.',
        name: 'ghostr.video.source',
        error: error,
        stackTrace: stackTrace,
      );
    }
    return _fallback.loadRemoteFeed(
      creatorIds: creatorIds,
      searchQuery: searchQuery,
      hashtags: hashtags,
    );
  }
}
