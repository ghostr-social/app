import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/src/rust/video/video.dart';

bool ffiVideoMatchesCanonical(
  FfiVideoDownload video,
  VideoPost post,
) {
  final reference = post.nostrReference;
  return reference != null &&
      _matchesReference(video.event, reference) &&
      _matchesDelivery(video, post.media) &&
      _matchesLocation(video, post.media) &&
      _matchesDigest(video, post.media);
}

bool _matchesDelivery(FfiVideoDownload video, VideoMediaSource canonical) {
  return video.nostr.delivery.name == canonical.remoteDelivery?.name;
}

bool _matchesLocation(FfiVideoDownload video, VideoMediaSource canonical) {
  return video.url == video.nostr.url &&
      canonical.remoteUrls.contains(video.url);
}

bool _matchesDigest(FfiVideoDownload video, VideoMediaSource canonical) {
  final digest = canonical.expectedSha256;
  return digest == null || video.nostr.id == digest.value;
}

bool _matchesReference(
  FfiNostrEventIdentity event,
  NostrEventReference reference,
) {
  return event.eventId == reference.eventId.value &&
      event.authorPublicKeyHex == reference.authorPublicKeyHex.value &&
      event.kind.toInt() == reference.kind.value &&
      event.identifier == reference.identifier?.value;
}

bool ffiVideoCanMapWithoutSnapshot(FfiVideoDownload video) {
  if (video.url != video.nostr.url) return false;
  if (video.nostr.delivery == FfiVideoDelivery.hls) return true;
  return video.localPath?.trim().isNotEmpty == true;
}

VideoMediaSource retainCanonicalVideoCacheMetadata(
  VideoMediaSource overlay,
  VideoMediaSource canonical,
) {
  var retained = overlay;
  final digest = canonical.expectedSha256;
  if (digest != null) {
    retained = VideoMediaSource.withExpectedSha256(retained, digest.value);
  }
  final scope = canonical.cacheScope;
  return scope == null
      ? retained
      : VideoMediaSource.withCacheScope(retained, scope.value);
}
