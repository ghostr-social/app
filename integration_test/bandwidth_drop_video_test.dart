import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_qoe_targets.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('bandwidth drop keeps active video inside the QoE budget', (
    tester,
  ) async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.bandwidthDrop,
    );
    addTearDown(testbed.close);

    final focus = await testbed.show(tester, 'bandwidth');
    await testbed.waitForPlaying(tester, focus);
    await testbed.waitForPosition(
      tester,
      focus.videoId,
      const Duration(seconds: 4),
    );

    expect(testbed.probe.playingLatency(focus), lessThan(deviceStartupTarget));
    expect(
      testbed.probe.rebufferRatio,
      lessThanOrEqualTo(deviceRebufferTarget),
    );
    expect(testbed.server.impairedResponses, greaterThan(0));
    expectNoPlaybackError(tester);
  });
}
