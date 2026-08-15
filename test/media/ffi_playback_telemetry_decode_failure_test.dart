import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

import '../support/ffi_playback_telemetry_fixture.dart';

void main() {
  test(
    'keeps decoder failure evidence before synthesized inactivity',
    () async {
      final first = Completer<void>();
      final sent = <FfiPlaybackObservation>[];
      final telemetry = FfiPlaybackTelemetryPort(
        reportPlayback: ({required input}) {
          sent.add(input);
          return sent.length == 1 ? first.future : Future.value();
        },
      );
      final session = telemetrySession('clip', 1);
      telemetry.activate(session);
      telemetry.report(telemetryObservation(session, PlaybackPhase.starting));
      telemetry.report(telemetryObservation(session, PlaybackPhase.failed));
      telemetry.deactivate(session);

      first.complete();
      await drainTelemetryMicrotasks();

      expect(sent.map((item) => item.phase), [
        FfiPlaybackPhase.starting,
        FfiPlaybackPhase.failed,
        FfiPlaybackPhase.inactive,
      ]);
    },
  );
}
