import 'package:ghostr/core/media/playback_delivery_id.dart';
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
    delivery: _delivery(delivery),
    sha256: media.expectedSha256?.value,
    sizeBytes: _bigInt(metadata.sizeBytes),
    durationMs: _bigInt(metadata.durationMs),
    blurhash: metadata.blurhash,
  );
}

FfiMediaDelivery _delivery(VideoMediaDelivery delivery) {
  return switch (delivery) {
    VideoMediaDelivery.progressive => FfiMediaDelivery.progressive,
    VideoMediaDelivery.hls => FfiMediaDelivery.hls,
  };
}

String ffiPostIdForMedia(VideoMediaSource media) {
  final deliveryId = media.playbackDeliveryId;
  if (deliveryId == null) {
    throw ArgumentError.value(media, 'media', 'Remote media is required.');
  }
  return deliveryId.value;
}

BigInt? _bigInt(int? value) => value == null ? null : BigInt.from(value);
