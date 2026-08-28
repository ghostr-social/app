import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_progressive_playback_gateway.dart';
import '../support/feed_preparation_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  testWidgets('exact preparation keeps the current native player', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    final currentUrl = fixture.url('p0');
    final gateway = FakeProgressivePlaybackGateway(
      immediatePlaybackUrl: currentUrl,
    );
    final hls = FakeHlsPlaybackGateway();
    final frames = StreamController<Object?>();
    final feedback = RecordingPlayerPreparationFeedback();
    final telemetry = RecordingPlaybackTelemetryPort();
    final rendered = NativeRenderedFirstFramePort(events: frames.stream);
    final playback = HlsVideoPlaybackPort(
      gateway: hls,
      delegate: GatewayVideoPlaybackPort(
        delegate: VideoPlayerPlaybackPort(
          telemetry: telemetry,
          preparationFeedback: feedback,
          renderedFirstFrames: rendered,
        ),
        gateway: gateway,
      ),
    );
    await fixture.pump(tester, playbackPort: playback);
    final source = fixture.platform.sources.values.single;
    final token = source.httpHeaders[warpPlaybackAttemptHeader];

    expect(fixture.platform.creationsFor(currentUrl), 1);
    expect(token, isNotNull);
    fixture.publish(1, 'p0', null);
    await _turn(tester);
    frames.add({'version': 1, 'attemptToken': token});
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();

    expect(gateway.requests, hasLength(1));
    expect(hls.requests, isEmpty);
    expect(fixture.platform.creationsFor(currentUrl), 1);
    expect(fixture.platform.playerCount, 1);
    expect(feedback.events.map((event) => event.state), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
      RecordedPreparationState.firstFrameRendered,
    ]);
    expect(telemetry.presentations, hasLength(1));
  });
}

Future<void> _turn(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await tester.pump();
}
