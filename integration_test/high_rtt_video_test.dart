import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_qoe_targets.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('high RTT starts and plays inside the QoE budget', (
    tester,
  ) async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.highRtt,
    );
    addTearDown(testbed.close);

    final focus = await testbed.show(tester, 'high-rtt');
    await testbed.waitForPlaying(tester, focus);
    await testbed.waitForPosition(
      tester,
      focus.videoId,
      const Duration(seconds: 3),
    );

    expect(testbed.probe.playingLatency(focus), lessThan(deviceStartupTarget));
    expect(
      testbed.probe.firstFrameLatency(focus),
      lessThan(deviceStartupTarget),
    );
    expect(
      testbed.probe.rebufferRatio,
      lessThanOrEqualTo(deviceRebufferTarget),
    );
    expect(testbed.server.impairedResponses, greaterThanOrEqualTo(4));
    expectNoPlaybackError(tester);
  });
}
