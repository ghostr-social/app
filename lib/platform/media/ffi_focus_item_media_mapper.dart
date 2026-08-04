import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';

/// Maps playable media onto the FFI focus-item payload. Focus updates
/// and playback-URL resolution must share this mapping so a post id
/// always addresses the same partial-store entry in the Rust engine.
FfiFocusItem ffiFocusItemForMedia(VideoMediaSource media) {
  final delivery = media.remoteDelivery;
  final urls = media.cacheSourceUrls;
  if (delivery == null || urls.isEmpty) {
    throw ArgumentError.value(media, 'media', 'Remote media is required.');
  }
  final metadata = media.mediaMetadata;
  return FfiFocusItem(
    postId: ffiPostIdForMedia(media),
    urls: urls,
    delivery: delivery.name,
    sha256: media.expectedSha256?.value,
    sizeBytes: _bigInt(metadata.sizeBytes),
    durationMs: _bigInt(metadata.durationMs),
  );
}

/// Stable gateway id for one post: the nostr event scope when it is
/// store-safe, the expected digest otherwise, and a digest of the
/// primary URL as the last resort so ids survive restarts.
String ffiPostIdForMedia(VideoMediaSource media) {
  final scope = media.cacheScope?.value;
  if (scope != null && _storeSafeIdPattern.hasMatch(scope)) return scope;
  final digest = media.expectedSha256?.value;
  if (digest != null) return digest;
  return _urlDigest(media);
}

final _storeSafeIdPattern = RegExp(r'^[A-Za-z0-9_-]+$');

String _urlDigest(VideoMediaSource media) {
  final url = media.remoteUrl;
  if (url == null) {
    throw ArgumentError.value(media, 'media', 'Remote media is required.');
  }
  return 'url-${sha256.convert(utf8.encode(url))}';
}

BigInt? _bigInt(int? value) => value == null ? null : BigInt.from(value);
