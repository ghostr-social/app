import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_event_matcher.dart';
import 'package:ghostr/src/rust/video/video.dart';

class FfiVideoMediaSourceMapper {
  const FfiVideoMediaSourceMapper();

  VideoMediaSource overlay(
    FfiVideoDownload video,
    VideoMediaSource canonical,
  ) {
    final remote = _remote(video, canonical.remoteUrls);
    final candidate = _importCandidate(video, remote);
    return retainCanonicalVideoCacheMetadata(candidate, canonical);
  }

  VideoMediaSource native(FfiVideoDownload video) {
    var source = _importCandidate(video, _remote(video));
    final expectedDigest = video.nostr.expectedDigest;
    if (expectedDigest != null) {
      source = VideoMediaSource.withExpectedSha256(source, expectedDigest);
    }
    return VideoMediaSource.withCacheScope(source, video.event.eventId);
  }

  VideoMediaSource _remote(
    FfiVideoDownload video, [
    Iterable<String> preferredFallbacks = const [],
  ]) {
    final fallbacks = <String>{
      ...preferredFallbacks,
      ...video.nostr.fallbackUrls,
    }..remove(video.url);
    return VideoMediaSource.remote(
      video.url,
      fallbackUrls: fallbacks.take(maxVideoCacheSourceCount - 1).toList(),
      delivery: _delivery(video.nostr.delivery),
    );
  }

  VideoMediaSource _importCandidate(
    FfiVideoDownload video,
    VideoMediaSource remote,
  ) {
    final sourcePath = video.localPath?.trim();
    if (sourcePath == null ||
        sourcePath.isEmpty ||
        remote.remoteDelivery != VideoMediaDelivery.progressive) {
      return remote;
    }
    return VideoMediaSource.importable(
      sourcePath,
      remoteUrl: remote.remoteUrl!,
      fallbackUrls: remote.fallbackUrls,
    );
  }

  VideoMediaDelivery _delivery(FfiVideoDelivery delivery) {
    return switch (delivery) {
      FfiVideoDelivery.progressive => VideoMediaDelivery.progressive,
      FfiVideoDelivery.hls => VideoMediaDelivery.hls,
    };
  }
}
