import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_bandwidth_recovery_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'real Android feed adapts to shared bandwidth loss and recovery',
    runWarpBandwidthRecoveryScenario,
  );
}
