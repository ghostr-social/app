import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_lease.dart';

const maxHlsPlaybackSourceCount = 5;

abstract interface class HlsPlaybackGatewayPort {
  Future<HlsPlaybackLease> acquire(HlsPlaybackRequest request);
}

final class HlsPlaybackRequest {
  HlsPlaybackRequest._(this.sourceUrls);

  factory HlsPlaybackRequest.fromMedia(VideoMediaSource media) {
    if (!_isCanonicalHls(media)) {
      throw ArgumentError.value(
          media, 'media', 'Remote HLS media is required.');
    }
    final sources = media.remoteUrls
        .take(maxHlsPlaybackSourceCount)
        .map(_validatedSourceUri)
        .toList(growable: false);
    return HlsPlaybackRequest._(List<Uri>.unmodifiable(sources));
  }

  final List<Uri> sourceUrls;
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
