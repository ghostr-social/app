import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_player_verified_rescue_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'blocked playback preserves intent without a prepared rescue candidate',
    runWarpPlayerVerifiedRescueScenario,
  );
}
