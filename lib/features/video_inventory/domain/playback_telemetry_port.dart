import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';

/// Non-blocking playback telemetry ordered by session lifecycle.
///
/// Implementations may replace an unsent observation with a newer observation
/// from the same session. Activation and deactivation must remain ordered, and
/// a presented frame must stay separate from observation coalescing. No
/// implementation may throw into playback.
abstract interface class PlaybackTelemetryPort {
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  );

  void activate(PlaybackSession session);

  void report(PlaybackObservation observation);

  void presented(PlaybackSession session);

  void deactivate(PlaybackSession session);
}

final class NoopPlaybackTelemetryPort implements PlaybackTelemetryPort {
  const NoopPlaybackTelemetryPort();

  @override
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  ) {
    return PlaybackSession(videoId, deliveryId, 1);
  }

  @override
  void activate(PlaybackSession session) {}

  @override
  void report(PlaybackObservation observation) {}

  @override
  void presented(PlaybackSession session) {}

  @override
  void deactivate(PlaybackSession session) {}
}
