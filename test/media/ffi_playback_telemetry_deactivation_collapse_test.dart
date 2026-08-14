import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

void main() {
  test('closed session keeps only its terminal queued sample', () async {
    final first = Completer<void>();
    final sent = <FfiPlaybackObservation>[];
    final telemetry = FfiPlaybackTelemetryPort(
      reportPlayback: ({required input}) {
        sent.add(input);
        return sent.length == 1 ? first.future : Future.value();
      },
    );
    final session = PlaybackSession(
      PlaybackVideoId.parse('clip'),
      PlaybackDeliveryId.parse('delivery'),
      1,
    );
    telemetry.activate(session);
    for (final phase in PlaybackPhase.values.take(4)) {
      telemetry.report(_observation(session, phase));
    }
    telemetry.report(_observation(session, PlaybackPhase.inactive));
    telemetry.deactivate(session);

    first.complete();
    await _drainMicrotasks();

    expect(sent.map((item) => item.phase), [
      FfiPlaybackPhase.starting,
      FfiPlaybackPhase.inactive,
    ]);
  });
}

PlaybackObservation _observation(PlaybackSession session, PlaybackPhase phase) {
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

Future<void> _drainMicrotasks() async {
  for (var index = 0; index < 5; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
