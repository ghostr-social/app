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
    ProxiedHlsVideoMediaSource(:final playbackUri) => PlaybackDeliveryId.parse(
      playbackUri.pathSegments[1],
    ),
    ProxiedProgressiveVideoMediaSource(:final playbackUri) =>
      PlaybackDeliveryId.parse(playbackUri.queryParameters['id']!),
    _ => null,
  };
}
