import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';

void main() {
  test('default telemetry accepts a complete playback lifecycle', () {
    const telemetry = NoopPlaybackTelemetryPort();
    final session = telemetry.activate(PlaybackVideoId.parse('clip'));

    telemetry.report(
      PlaybackObservation(
        session: session,
        phase: PlaybackPhase.playing,
        metrics: PlaybackMetrics(
          position: Duration.zero,
          bufferedExtent: const Duration(seconds: 2),
          playbackRate: 1,
        ),
      ),
    );
    telemetry.deactivate(session);

    expect(session.videoId, PlaybackVideoId.parse('clip'));
  });
}
