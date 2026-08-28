import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/media/video_media_source.dart';

final class PlaybackDeliveryId {
  factory PlaybackDeliveryId.parse(String raw) {
    final value = raw.trim();
    if (value.isEmpty) {
      throw const FormatException('A playback delivery id is required.');
    }
    return PlaybackDeliveryId._(value);
  }

  const PlaybackDeliveryId._(this.value);

  final String value;

  @override
  bool operator ==(Object other) {
    return other is PlaybackDeliveryId && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}

extension VideoMediaPlaybackDelivery on VideoMediaSource {
  PlaybackDeliveryId? get playbackDeliveryId => switch (this) {
    ProxiedHlsVideoMediaSource() => null,
    ProxiedProgressiveVideoMediaSource(:final playbackUri) =>
      PlaybackDeliveryId.parse(playbackUri.queryParameters['id']!),
    _ when remoteUrl != null => PlaybackDeliveryId.parse(
      _remoteDeliveryId(this),
    ),
    _ => null,
  };
}

String _remoteDeliveryId(VideoMediaSource media) {
  final scope = media.cacheScope?.value;
  if (scope != null && _storeSafeIdPattern.hasMatch(scope)) return scope;
  final digest = media.expectedSha256?.value;
  if (digest != null) return digest;
  return 'url-${sha256.convert(utf8.encode(media.remoteUrl!))}';
}

final _storeSafeIdPattern = RegExp(r'^[A-Za-z0-9_-]+$');
