import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';

final class RecordingPlaybackTelemetryPort implements PlaybackTelemetryPort {
  final activations = <PlaybackSession>[];
  final observations = <PlaybackObservation>[];
  final presentations = <PlaybackSession>[];
  final deactivations = <PlaybackSession>[];
  var _nextGeneration = 0;

  @override
  PlaybackSession openSession(
    PlaybackVideoId videoId,
    PlaybackDeliveryId deliveryId,
  ) {
    return PlaybackSession(videoId, deliveryId, ++_nextGeneration);
  }

  @override
  void activate(PlaybackSession session) {
    activations.add(session);
  }

  @override
  void report(PlaybackObservation observation) {
    observations.add(observation);
  }

  @override
  void presented(PlaybackSession session) {
    presentations.add(session);
  }

  @override
  void deactivate(PlaybackSession session) {
    deactivations.add(session);
  }
}
