import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_ready_burst_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'physical feed replays retained bytes within the two-player limit',
    runWarpAdaptiveWarmBackScenario,
  );
}
