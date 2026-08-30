import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';

const maxHlsPlaybackSourceCount = 5;

abstract interface class HlsPlaybackGatewayPort {
  Future<HlsPlaybackLease> acquire(HlsPlaybackRequest request);
}

final class HlsPlaybackRequest {
  HlsPlaybackRequest._(
    this.deliveryId,
    this.representationId,
    this.expectedAuthority,
    this.sourceUrls,
  );

  factory HlsPlaybackRequest.fromMedia(
    VideoMediaSource media, {
    HlsPlaybackAuthority? expectedAuthority,
  }) {
    if (!_isCanonicalHls(media)) {
      throw ArgumentError.value(
        media,
        'media',
        'Remote HLS media is required.',
      );
    }
    final deliveryId = media.playbackDeliveryId;
    if (deliveryId == null) {
      throw ArgumentError.value(
        media,
        'media',
        'Delivery identity is required.',
      );
    }
    final representationId = VideoRepresentationId.forMedia(media);
    _validateExpectedAuthority(expectedAuthority, deliveryId, representationId);
    final sources = media.remoteUrls
        .take(maxHlsPlaybackSourceCount)
        .map(_validatedSourceUri)
        .toList(growable: false);
    return HlsPlaybackRequest._(
      deliveryId,
      representationId,
      expectedAuthority,
      List<Uri>.unmodifiable(sources),
    );
  }

  final PlaybackDeliveryId deliveryId;
  final VideoRepresentationId representationId;
  final HlsPlaybackAuthority? expectedAuthority;
  final List<Uri> sourceUrls;
}

void _validateExpectedAuthority(
  HlsPlaybackAuthority? authority,
  PlaybackDeliveryId deliveryId,
  VideoRepresentationId representationId,
) {
  if (authority == null) return;
  if (authority.deliveryId != deliveryId ||
      authority.representationId != representationId) {
    throw ArgumentError.value(
      authority,
      'expectedAuthority',
      'Must match the HLS media identity.',
    );
  }
}

bool _isCanonicalHls(VideoMediaSource media) {
  return !media.isLocal &&
      media is! ProxiedHlsVideoMediaSource &&
      media.remoteDelivery == VideoMediaDelivery.hls;
}

Uri _validatedSourceUri(String raw) {
  final uri = Uri.parse(raw);
  final isHttp = uri.scheme == 'http' || uri.scheme == 'https';
  if (!isHttp || uri.host.isEmpty || uri.userInfo.isNotEmpty) {
    throw FormatException('Invalid HLS source URL: $raw');
  }
  return uri;
}
