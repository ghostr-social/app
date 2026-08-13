import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

void main() {
  test('a failed report does not poison later playback telemetry', () async {
    final sent = <FfiPlaybackObservation>[];
    final telemetry = FfiPlaybackTelemetryPort(
      reportPlayback: ({required input}) async {
        sent.add(input);
        if (sent.length == 1) throw StateError('delivery unavailable');
      },
    );
    final session = PlaybackSession(
      PlaybackVideoId.parse('clip'),
      PlaybackDeliveryId.parse('delivery'),
      1,
    );
    telemetry.activate(session);

    telemetry.report(observation(session, PlaybackPhase.starting));
    telemetry.report(observation(session, PlaybackPhase.playing));
    await drainMicrotasks();

    expect(sent.map((item) => item.phase), [
      FfiPlaybackPhase.starting,
      FfiPlaybackPhase.playing,
    ]);
  });
}

PlaybackObservation observation(PlaybackSession session, PlaybackPhase phase) {
  return PlaybackObservation(
    session: session,
    phase: phase,
    metrics: PlaybackMetrics(
      position: Duration.zero,
      bufferedExtent: Duration(seconds: 2),
      playbackRate: 1,
    ),
  );
}

Future<void> drainMicrotasks() async {
  for (var index = 0; index < 4; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
