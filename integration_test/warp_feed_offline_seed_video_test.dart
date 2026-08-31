import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_offline_restart_fixture.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('seeds a signed feed event and complete progressive bytes', (
    tester,
  ) async {
    final fixture = await WarpOfflineRestartFixture.seed();
    addTearDown(fixture.close);

    await tester.pumpWidget(fixture.app);
    fixture.load();
    await fixture.waitForCachedPost(tester);
    final focus = await fixture.waitForFocus(tester);
    await fixture.waitForFreshFrame(tester, focus);
    await fixture.waitForCompleteCurrentCache(tester, focus);
    await fixture.waitForDurableEventSnapshot(tester);

    expect(fixture.hasCachedSignedPost, isTrue);
    expect(fixture.relay!.eventsSent, greaterThan(0));
    expect(fixture.originBodyRequestedIds, contains('current'));
    expect(find.text('Video unavailable'), findsNothing);
    debugPrint(fixture.seedEvidence(focus));
  });
}
