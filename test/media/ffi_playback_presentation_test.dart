import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:ghostr/platform/media/ffi_playback_telemetry_port.dart';
import 'package:ghostr/src/rust/api/playback_types.dart';

void main() {
  test('presentation uses a dedicated ordered reporter', () async {
    final blocked = Completer<void>();
    final presented = <FfiPlaybackPresentation>[];
    final telemetry = FfiPlaybackTelemetryPort(
      reportPlayback: ({required input}) => blocked.future,
      reportPresentation: ({required input}) async => presented.add(input),
      presentationClock: () => 321,
    );
    final session = PlaybackSession(
      PlaybackVideoId.parse('clip'),
      PlaybackDeliveryId.parse('delivery'),
      7,
    );
    telemetry.activate(session);
    telemetry.report(
      PlaybackObservation(
        session: session,
        phase: PlaybackPhase.starting,
        metrics: PlaybackMetrics(
          position: Duration.zero,
          bufferedExtent: Duration.zero,
          playbackRate: 1,
        ),
      ),
    );

    telemetry.presented(session);
    await Future<void>.delayed(Duration.zero);

    expect(presented.single.postId, 'delivery');
    expect(presented.single.generation, BigInt.from(7));
    expect(presented.single.sequence, greaterThan(BigInt.zero));
    expect(presented.single.observedAtMs, BigInt.from(321));
    blocked.complete();
  });
}
