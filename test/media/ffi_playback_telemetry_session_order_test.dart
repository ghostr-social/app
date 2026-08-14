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
    'sends inactive before a third rapid session overtakes active playback',
    () async {
      final first = Completer<void>();
      final sent = <FfiPlaybackObservation>[];
      final telemetry = FfiPlaybackTelemetryPort(
        reportPlayback: ({required input}) {
          sent.add(input);
          return sent.length == 1 ? first.future : Future.value();
        },
      );
      final old = session('old', 1);
      telemetry.activate(old);
      telemetry.report(observation(old, PlaybackPhase.playing));
      telemetry.report(observation(old, PlaybackPhase.inactive));
      telemetry.deactivate(old);
      final skipped = session('skipped', 2);
      telemetry.activate(skipped);
      telemetry.report(observation(skipped, PlaybackPhase.starting));
      telemetry.report(observation(skipped, PlaybackPhase.inactive));
      telemetry.deactivate(skipped);
      final current = session('current', 3);
      telemetry.activate(current);
      telemetry.report(observation(current, PlaybackPhase.starting));

      first.complete();
      await drainMicrotasks();

      expect(sent.map((item) => item.postId), [
        'delivery-old',
        'delivery-old',
        'delivery-current',
      ]);
      expect(sent[1].phase, FfiPlaybackPhase.inactive);
      expect(sent[1].generation, lessThan(sent[2].generation));
    },
  );
}

PlaybackSession session(String videoId, int generation) {
  return PlaybackSession(
    PlaybackVideoId.parse(videoId),
    PlaybackDeliveryId.parse('delivery-$videoId'),
    generation,
  );
}

PlaybackObservation observation(PlaybackSession session, PlaybackPhase phase) {
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

Future<void> drainMicrotasks() async {
  for (var index = 0; index < 5; index += 1) {
    await Future<void>.delayed(Duration.zero);
  }
}
