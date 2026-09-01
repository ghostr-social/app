import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_stale_validator_rotation_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'Android rejects stale bytes and decodes the replacement generation',
    runWarpStaleValidatorRotationScenario,
  );
}
