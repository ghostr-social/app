import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_long_session_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'physical feed stays bounded through 32 handoffs across 24 posts',
    (tester) async {
      final scenario = await WarpLongSessionScenario.start();
      addTearDown(scenario.close);

      await scenario.run(tester);
    },
  );
}
