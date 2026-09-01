import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_invalid_track_fallback_scenario.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'definitive player rejection selects a decoded rendition without a swipe',
    (tester) async {
      final scenario = await WarpInvalidTrackFallbackScenario.start();
      addTearDown(scenario.close);

      await scenario.run(tester);
    },
  );
}
