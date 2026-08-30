import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_mixed_feed_readiness_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('mixed WARP feed presents HLS without structural-only rescue', (
    tester,
  ) async {
    await runWarpMixedFeedReadinessScenario(tester);
  });
}
