import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('prepared predecessor activates with a newer generation', (
    tester,
  ) async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.contract,
    );
    final port = VideoPlayerPlaybackPort(telemetry: testbed.probe);
    addTearDown(testbed.close);

    final currentFocus = await showPair(tester, testbed, port, activeIndex: 1);
    await testbed.waitForPlaying(tester, currentFocus);
    final current = testbed.probe.activations.single;
    final previousFocus = await showPair(tester, testbed, port, activeIndex: 0);
    await testbed.waitForPlaying(tester, previousFocus);

    expect(
      testbed.probe.activations.last.generation,
      greaterThan(current.generation),
    );
  });
}

Future<PlaybackFocus> showPair(
  WidgetTester tester,
  DevicePlaybackTestbed testbed,
  VideoPlayerPlaybackPort port, {
  required int activeIndex,
}) async {
  final activeId = PlaybackVideoId.parse('prepared-$activeIndex');
  final focus = testbed.probe.markFocus(activeId);
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: List.generate(2, (index) {
          final id = PlaybackVideoId.parse('prepared-$index');
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: ProxiedHlsVideoMediaSource(
                testbed.server.playbackUri(id.value).toString(),
              ),
              videoId: id,
              isActive: index == activeIndex,
            ),
          );
        }),
      ),
    ),
  );
  return focus;
}
