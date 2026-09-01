import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_ignored_range_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'ignored Range reaches a decoded frame with bounded rescue',
    runWarpIgnoredRangeScenario,
  );
}
