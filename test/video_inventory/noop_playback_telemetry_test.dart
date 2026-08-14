import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_telemetry_port.dart';

void main() {
  test('default telemetry accepts a complete playback lifecycle', () {
    final createTelemetry = NoopPlaybackTelemetryPort.new;
    final telemetry = createTelemetry();
    final session = telemetry.openSession(
      PlaybackVideoId.parse('clip'),
      PlaybackDeliveryId.parse('delivery'),
    );
    telemetry.activate(session);

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
