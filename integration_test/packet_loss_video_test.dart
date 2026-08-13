import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_qoe_targets.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a deterministic disconnect retries without user intervention', (
    tester,
  ) async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.packetLoss,
    );
    addTearDown(testbed.close);

    final focus = await testbed.show(tester, 'packet-loss');
    await testbed.waitForPlaying(tester, focus);
    await testbed.waitUntil(tester, () => testbed.server.disconnects == 1);
    final positionAtLoss = testbed.probe.latestPosition(focus.videoId);
    await testbed.waitForPosition(
      tester,
      focus.videoId,
      const Duration(seconds: 4),
    );

    expect(testbed.server.requestsFor('index2.m4s'), greaterThanOrEqualTo(2));
    expect(
      testbed.probe.latestPosition(focus.videoId),
      greaterThanOrEqualTo(positionAtLoss),
    );
    expect(
      testbed.probe.rebufferRatio,
      lessThanOrEqualTo(deviceRebufferTarget),
    );
    expectNoPlaybackError(tester);
  });
}
