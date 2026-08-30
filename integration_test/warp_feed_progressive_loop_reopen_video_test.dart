import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_progressive_loop_reopen_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'stable progressive promotion survives the exact Android loop reopen',
    runWarpProgressiveLoopReopenScenario,
  );
}
