import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';

void main() {
  test('settled waits for observation and presentation reporters', () async {
    final observations = Completer<void>();
    final lateObservation = Completer<void>();
    final presentations = Completer<void>();
    var observationCalls = 0;
    final telemetry = FfiPlaybackTelemetryPort(
      reportPlayback: ({required input}) {
        observationCalls += 1;
        return observationCalls == 1
            ? observations.future
            : lateObservation.future;
      },
      reportPresentation: ({required input}) => presentations.future,
    );
    final session = PlaybackSession(
      PlaybackVideoId.parse('clip'),
      PlaybackDeliveryId.parse('delivery'),
      1,
    );
    telemetry.activate(session);
    telemetry.report(_playing(session));
    telemetry.presented(session);
    var didSettle = false;
    final settling = telemetry.settled.then((_) => didSettle = true);

    await Future<void>.delayed(Duration.zero);
    expect(didSettle, isFalse);
    telemetry.report(_playing(session));
    observations.complete();
    await _drainMicrotasks();
    expect(observationCalls, 2);
    expect(didSettle, isFalse);
    presentations.complete();
    await settling;
    expect(didSettle, isTrue);
    lateObservation.complete();
    await telemetry.settled;
  });
}

Future<void> _drainMicrotasks() async {
  for (var index = 0; index < 4; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}

PlaybackObservation _playing(PlaybackSession session) {
  return PlaybackObservation(
    session: session,
    phase: PlaybackPhase.playing,
    metrics: PlaybackMetrics(
      position: Duration.zero,
      bufferedExtent: const Duration(seconds: 2),
      playbackRate: 1,
    ),
  );
}
