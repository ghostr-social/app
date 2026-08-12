import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';

final class RecordingPlaybackTelemetryPort implements PlaybackTelemetryPort {
  final activations = <PlaybackSession>[];
  final observations = <PlaybackObservation>[];
  final deactivations = <PlaybackSession>[];
  var _nextGeneration = 0;

  @override
  PlaybackSession activate(PlaybackVideoId videoId) {
    final session = PlaybackSession(videoId, ++_nextGeneration);
    activations.add(session);
    return session;
  }

  @override
  void report(PlaybackObservation observation) {
    observations.add(observation);
  }

  @override
  void deactivate(PlaybackSession session) {
    deactivations.add(session);
  }
}
