import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_evidence_models.dart';
import 'support/warp_feed_playback_journey.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('signed Android feed plans with its real Wi-Fi generation', (
    tester,
  ) async {
    final journey = await WarpFeedPlaybackJourney.start();
    addTearDown(journey.close);
    await tester.pumpWidget(journey.app);
    journey.load();
    await journey.waitForCaption(tester, 0);
    await journey.waitForPublishedFocus(tester, 0);

    final plan = await journey.waitForPlan(
      tester,
      (plan) =>
          plan.networkStatusGeneration == 1 &&
          plan.networkClass == WarpNetworkClass.wifi &&
          plan.focusGeneration != null &&
          plan.plan.readyReserve.candidateCount >= 1,
    );

    expect(plan.focusGeneration, isNotNull);
    expect(plan.plan.readyReserve.target, greaterThanOrEqualTo(1));
    expect(plan.plan.readyReserve.candidateCount, greaterThanOrEqualTo(1));
  });
}
