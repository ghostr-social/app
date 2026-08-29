import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

import '../support/ffi_playback_telemetry_fixture.dart';

void main() {
  test('late predecessor close still collapses its queued samples', () async {
    final first = Completer<void>();
    final sent = <FfiPlaybackObservation>[];
    final telemetry = FfiPlaybackTelemetryPort(
      reportPlayback: ({required input}) {
        sent.add(input);
        return sent.length == 1 ? first.future : Future.value();
      },
    );
    final old = telemetrySession('old', 1);
    telemetry.activate(old);
    telemetry.report(telemetryObservation(old, PlaybackPhase.starting));
    telemetry.report(telemetryObservation(old, PlaybackPhase.playing));
    final current = telemetrySession('current', 2);
    telemetry.activate(current);
    telemetry.report(telemetryObservation(current, PlaybackPhase.starting));

    telemetry.deactivate(old);
    first.complete();
    await telemetry.settled;

    expect(sent.map((item) => item.phase), [
      FfiPlaybackPhase.starting,
      FfiPlaybackPhase.inactive,
      FfiPlaybackPhase.starting,
    ]);
  });
}
