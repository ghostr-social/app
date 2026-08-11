import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_qoe_targets.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'rapid swipes give the final focus prompt uninterrupted playback',
    (tester) async {
      final testbed = await DevicePlaybackTestbed.start(
        DeviceVideoScenario.rapidSwipes,
      );
      addTearDown(testbed.close);

      await testbed.show(tester, 'first');
      await testbed.pumpFor(tester, const Duration(milliseconds: 200));
      await testbed.show(tester, 'second');
      await testbed.pumpFor(tester, const Duration(milliseconds: 200));
      await testbed.show(tester, 'first');
      await testbed.pumpFor(tester, const Duration(milliseconds: 200));
      final finalFocus = await testbed.show(tester, 'second');
      await testbed.waitForPlaying(tester, finalFocus);

      expect(
        testbed.probe.playingLatency(finalFocus),
        lessThan(deviceFocusSwitchTarget),
      );
      expect(
        testbed.server.requestedSessions,
        containsAll(['first', 'second']),
      );
      expect(
        testbed.server.cancellationWasteBytes,
        lessThanOrEqualTo(192 * 1024),
      );
      expectNoPlaybackError();
    },
  );
}
