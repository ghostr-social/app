import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

void main() {
  test(
    'coalesces samples without dropping stall boundary transitions',
    () async {
      final first = Completer<void>();
      final sent = <FfiPlaybackObservation>[];
      var calls = 0;
      final telemetry = FfiPlaybackTelemetryPort(
        reportPlayback: ({required input}) {
          sent.add(input);
          calls += 1;
          return calls == 1 ? first.future : Future.value();
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
      telemetry.report(observation(session, PlaybackPhase.networkStalled));
      telemetry.report(observation(session, PlaybackPhase.networkStalled));
      telemetry.report(observation(session, PlaybackPhase.playing));
      expect(sent.single.phase, FfiPlaybackPhase.starting);

      first.complete();
      await drainMicrotasks();

      expect(sent.map((item) => item.phase), [
        FfiPlaybackPhase.starting,
        FfiPlaybackPhase.playing,
        FfiPlaybackPhase.networkStalled,
        FfiPlaybackPhase.playing,
      ]);
      expect(sent[2].sequence, BigInt.from(4));
      expect(sent.last.sequence, BigInt.from(5));
    },
  );
}

PlaybackObservation observation(PlaybackSession session, PlaybackPhase phase) {
  return PlaybackObservation(
    session: session,
    phase: phase,
    metrics: PlaybackMetrics(
      position: const Duration(seconds: 2),
      bufferedExtent: const Duration(seconds: 5),
      playbackRate: 1.25,
    ),
  );
}

Future<void> drainMicrotasks() async {
  for (var index = 0; index < 4; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
