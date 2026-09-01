import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_origin_timeout_fallback_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'stalled primary yields to its decoded fallback at the origin deadline',
    runWarpOriginTimeoutFallbackScenario,
  );
}
