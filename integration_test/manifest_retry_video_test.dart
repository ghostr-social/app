import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_qoe_targets.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('failed HLS manifest request recovers on the same URL', (
    tester,
  ) async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.manifestRetry,
    );
    addTearDown(testbed.close);

    final focus = await testbed.show(tester, 'manifest-retry');
    await testbed.waitForPlaying(tester, focus);
    await testbed.waitForPosition(
      tester,
      focus.videoId,
      const Duration(seconds: 2),
    );

    expect(testbed.server.manifestFailures, 1);
    expect(testbed.server.successfulManifestResponses, greaterThan(0));
    expect(testbed.server.requestsFor('index.m3u8'), greaterThanOrEqualTo(2));
    expect(
      testbed.probe.playingLatency(focus),
      lessThan(deviceManifestRetryStartupTarget),
    );
    expectNoPlaybackError();
  });
}
