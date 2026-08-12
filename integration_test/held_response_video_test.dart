import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_qoe_targets.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'held media response resumes at the same playhead after release',
    (tester) async {
      final testbed = await DevicePlaybackTestbed.start(
        DeviceVideoScenario.heldResponse,
      );
      addTearDown(testbed.close);

      final focus = await testbed.show(tester, 'held-response');
      await testbed.waitForPlaying(tester, focus);
      await testbed.waitUntil(tester, () => testbed.server.isResponseHeld);
      await testbed.waitForPhase(tester, PlaybackPhase.networkStalled);
      final stalledAt = testbed.probe.latestPosition(focus.videoId);
      final releasedAt = testbed.probe.elapsed;

      expect(testbed.server.heldResponses, greaterThan(0));
      testbed.server.releaseHeldResponse();
      await testbed.waitForPhase(
        tester,
        PlaybackPhase.playing,
        after: releasedAt,
      );

      expect(
        testbed.probe.recoveryLatency(releasedAt),
        lessThan(deviceHeldResponseRecoveryTarget),
      );
      expect(
        testbed.probe.latestPosition(focus.videoId),
        greaterThanOrEqualTo(stalledAt),
      );
      expectNoPlaybackError();
    },
  );
}
