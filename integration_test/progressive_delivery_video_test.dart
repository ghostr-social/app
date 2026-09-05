import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/progressive_device_journey.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('real progressive delivery starts and prepares the next video', (
    tester,
  ) async {
    final journey = await ProgressiveDeviceJourney.start();
    addTearDown(journey.close);

    await journey.focusCurrentAndNext();
    await journey.waitForPreparedNext(tester);
    await journey.showCurrent(tester);
    await journey.waitForCurrentFrame(tester);
    await journey.waitForAcceptedPlayback(tester, 1);
    await journey.waitForAcceptedDelivery(tester, 'a' * 64);
    final currentBytes = journey.currentOriginBytes;
    await journey.showNext(tester);
    await journey.waitForNextFrame(tester);
    await journey.waitForAcceptedPlayback(tester, 2);
    await journey.waitForAcceptedDelivery(tester, 'b' * 64);
    await journey.pumpFor(tester, const Duration(milliseconds: 500));
    final admissions = await journey.playbackAdmissions();

    expect(journey.playbackBypassedHeadResponses, isTrue);
    expect(
      journey.currentOriginBytes - currentBytes,
      lessThanOrEqualTo(192 * 1024),
    );
    expect(journey.completedRangesDoNotOverlap, isTrue);
    expect(
      journey.submittedPlaybackDeliveryIds,
      containsAll(['a' * 64, 'b' * 64]),
    );
    expect(admissions.accepted, greaterThanOrEqualTo(BigInt.from(2)));
    expect(admissions.lastAcceptedDeliveryId, 'b' * 64);
    expect(admissions.inactiveDelivery, BigInt.zero);
    expect(admissions.staleSession, BigInt.zero);
    expect(admissions.staleSequence, BigInt.zero);
    expect(journey.hasPlaybackError(tester), isFalse);
  });
}
