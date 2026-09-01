import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_android_lifecycle_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('physical HOME releases and foreground decodes the feed', (
    tester,
  ) async {
    final scenario = await WarpAndroidLifecycleScenario.start();
    addTearDown(scenario.close);
    await scenario.run(tester);
  });
}
