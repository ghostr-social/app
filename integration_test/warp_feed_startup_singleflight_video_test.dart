import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_startup_singleflight_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'real Android startup does not restart the next reserve prefix',
    runWarpStartupSingleflightScenario,
  );
}
