import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';

import '../support/fake_progressive_playback_gateway.dart';
import '../support/feed_preparation_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_player_preparation_feedback.dart';

void main() {
  testWidgets('three Ready feed players present through 200 ms swipes', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 6);
    final frames = StreamController<Object?>();
    final telemetry = RecordingPlaybackTelemetryPort();
    addTearDown(fixture.updates.close);
    addTearDown(frames.close);
    final playback = GatewayVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(
        telemetry: telemetry,
        preparationFeedback: RecordingPlayerPreparationFeedback(),
        renderedFirstFrames: NativeRenderedFirstFramePort(
          events: frames.stream,
        ),
      ),
      gateway: FakeProgressivePlaybackGateway(
        immediatePlaybackUrl: fixture.url('p0'),
      ),
    );
    await fixture.pump(tester, playbackPort: playback);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3']);
    await fixture.settle(tester);
    _renderPreparedFrames(fixture, frames);
    await fixture.settle(tester);

    await _swipeBurst(tester, 3);
    await fixture.settle(tester);

    expect(telemetry.activations.map((item) => item.videoId.value), [
      'p0',
      'p1',
      'p2',
      'p3',
    ]);
    expect(telemetry.presentations, telemetry.activations);
    for (final id in ['p1', 'p2', 'p3']) {
      expect(fixture.platform.creationsFor(fixture.url(id)), 1);
    }
  });
}

void _renderPreparedFrames(
  FeedPreparationFixture fixture,
  StreamController<Object?> frames,
) {
  for (final source in fixture.platform.sources.values) {
    final token = source.httpHeaders[warpPlaybackAttemptHeader];
    if (token != null) frames.add({'version': 1, 'attemptToken': token});
  }
}

Future<void> _swipeBurst(WidgetTester tester, int count) async {
  for (var index = 0; index < count; index += 1) {
    final page = find.byType(PageView);
    final height = tester.getSize(page).height;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, -height * 0.23));
    await tester.pump(const Duration(milliseconds: 16));
    await gesture.up();
    await tester.pump(const Duration(milliseconds: 184));
  }
}
