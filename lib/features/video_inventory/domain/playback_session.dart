import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';

final class PlaybackSession {
  factory PlaybackSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
    int generation,
  ) {
    if (generation < 1) {
      throw ArgumentError.value(generation, 'generation');
    }
    return PlaybackSession._(videoId, deliveryId, generation);
  }

  const PlaybackSession._(this.videoId, this.deliveryId, this.generation);

  final PlaybackVideoId videoId;
  final PlaybackDeliveryId deliveryId;
  final int generation;

  @override
  bool operator ==(Object other) {
    return other is PlaybackSession &&
        other.videoId == videoId &&
        other.deliveryId == deliveryId &&
        other.generation == generation;
  }

  @override
  int get hashCode => Object.hash(videoId, deliveryId, generation);
}
