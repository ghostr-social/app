import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_malformed_range_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'malformed 206 bytes stay unready and manual navigation preserves order',
    runWarpMalformedRangeScenario,
  );
}
