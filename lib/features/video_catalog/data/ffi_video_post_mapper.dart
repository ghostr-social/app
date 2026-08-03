import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_event_matcher.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_media_source_mapper.dart';
import 'package:ghostr/features/video_catalog/data/ffi_native_video_post_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/src/rust/video/video.dart';

class FfiVideoPostMapper {
  const FfiVideoPostMapper();

  static const _mediaMapper = FfiVideoMediaSourceMapper();

  Iterable<VideoPost> map(
    List<FfiVideoDownload> nativeVideos,
    List<VideoPost> snapshot,
  ) sync* {
    final remaining = nativeVideos.toList();
    final emitted = <VideoPostId>{};
    yield* _mapCanonical(snapshot, remaining, emitted);
    yield* _mapRemaining(remaining, emitted);
  }

  Iterable<VideoPost> _mapCanonical(
    List<VideoPost> snapshot,
    List<FfiVideoDownload> remaining,
    Set<VideoPostId> emitted,
  ) sync* {
    for (final post in snapshot.where(_isCanonical)) {
      final native = _takeMatching(post, remaining);
      if (native == null || !emitted.add(post.id)) continue;
      yield post.withMedia(_overlayMedia(post, native));
    }
  }

  Iterable<VideoPost> _mapRemaining(
    List<FfiVideoDownload> remaining,
    Set<VideoPostId> emitted,
  ) sync* {
    for (final native in remaining) {
      final post = tryMapFfiNativeVideo(native, _mediaSource);
      if (post != null && emitted.add(post.id)) yield post;
    }
  }

  bool _isCanonical(VideoPost post) => post.nostrReference != null;

  VideoMediaSource _overlayMedia(VideoPost post, FfiVideoDownload video) {
    return _mediaMapper.overlay(video, post.media);
  }

  FfiVideoDownload? _takeMatching(
    VideoPost post,
    List<FfiVideoDownload> candidates,
  ) {
    final index = candidates.indexWhere(
      (video) => ffiVideoMatchesCanonical(video, post),
    );
    return index < 0 ? null : candidates.removeAt(index);
  }

  VideoMediaSource _mediaSource(FfiVideoDownload video) {
    return _mediaMapper.native(video);
  }
}
