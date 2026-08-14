import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

void main() {
  test(
    'reports the delivery identity when the social identity differs',
    () async {
      final sent = <FfiPlaybackObservation>[];
      final telemetry = FfiPlaybackTelemetryPort(
        reportPlayback: ({required input}) async => sent.add(input),
      );
      final session = telemetry.openSession(
        PlaybackVideoId.parse('social-event-id'),
        PlaybackDeliveryId.parse('rust-delivery-id'),
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
      await _drainMicrotasks();

      expect(sent.single.postId, 'rust-delivery-id');
    },
  );
}

Future<void> _drainMicrotasks() async {
  for (var index = 0; index < 3; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
