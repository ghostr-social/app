import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';

PlaybackSession telemetrySession(String id, int generation) {
  return PlaybackSession(
    PlaybackVideoId.parse(id),
    PlaybackDeliveryId.parse('delivery-$id'),
    generation,
  );
}

PlaybackObservation telemetryObservation(
  PlaybackSession session,
  PlaybackPhase phase,
) {
  return PlaybackObservation(
    session: session,
    phase: phase,
    metrics: PlaybackMetrics(
      position: Duration.zero,
      bufferedExtent: const Duration(seconds: 2),
      playbackRate: 1,
    ),
  );
}

Future<void> drainTelemetryMicrotasks() async {
  for (var index = 0; index < 6; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
