import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_bandwidth_recovery_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'impaired warm return presents and plays within its protected target',
    runWarpBandwidthWarmReturnScenario,
  );
}
