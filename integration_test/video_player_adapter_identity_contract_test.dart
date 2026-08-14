import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_playback_testbed.dart';
import 'support/device_video_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('adapter retires covered generation with delivery identity', (
    tester,
  ) async {
    final testbed = await DevicePlaybackTestbed.start(
      DeviceVideoScenario.contract,
    );
    addTearDown(testbed.close);

    final focus = await testbed.show(tester, 'identity');
    await testbed.waitForPlaying(tester, focus);
    final first = testbed.probe.activations.single;
    expect(first.deliveryId.value, testbed.server.deliveryIdFor('identity'));

    await testbed.show(tester, 'identity', isActive: false);
    expect(testbed.probe.deactivations, contains(first));
    final coveredCount = testbed.probe.observations.length;
    await testbed.pumpFor(tester, const Duration(milliseconds: 300));
    expect(testbed.probe.observations, hasLength(coveredCount));

    final replacementFocus = await testbed.show(tester, 'identity');
    await testbed.waitForPlaying(tester, replacementFocus);
    final replacement = testbed.probe.activations.last;
    expect(replacement.generation, greaterThan(first.generation));
    expect(replacement.deliveryId, first.deliveryId);
  });
}
