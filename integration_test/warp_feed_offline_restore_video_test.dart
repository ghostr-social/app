import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'support/warp_offline_restart_fixture.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('restores cached feed and decodes without relay or origin', (
    tester,
  ) async {
    final fixture = await WarpOfflineRestartFixture.restore();
    addTearDown(fixture.closeAndDelete);

    await fixture.expectRelayUnavailable();
    fixture.load();
    await fixture.waitForCachedPost(tester);
    fixture.expectNoStalePlayerReadiness();
    expect(fixture.hasCachedSignedPost, isTrue);
    expect(fixture.relay, isNull);

    await tester.pumpWidget(fixture.app);
    final focus = await fixture.waitForFocus(tester);
    await fixture.waitForFreshFrame(tester, focus);
    await fixture.expectNoOriginRequest();
    expect(find.text('WARP signed current'), findsOneWidget);
    expect(find.text('Video unavailable'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
    await fixture.waitForPlayerCleanup(tester, focus);
    debugPrint(await fixture.restoreEvidence(focus));
  });
}
