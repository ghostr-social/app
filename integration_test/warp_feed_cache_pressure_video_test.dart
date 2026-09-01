import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_cache_pressure_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('physical cache pressure evicts and refetches a cold post', (
    tester,
  ) async {
    final scenario = await WarpCachePressureScenario.start();
    addTearDown(scenario.close);

    await scenario.run(tester);
  });
}
