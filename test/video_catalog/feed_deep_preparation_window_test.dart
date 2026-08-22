import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('three future players stay prepared across rapid swipes', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 6);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3']);
    await fixture.settle(tester);

    expect(fixture.platform.playerCount, 4);
    for (final id in ['p1', 'p2', 'p3']) {
      expect(fixture.platform.creationsFor(fixture.url(id)), 1);
      await fixture.swipe(tester);
      expect(fixture.platform.creationsFor(fixture.url(id)), 1);
    }

    expect(fixture.platform.peakPlayerCount, 4);
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}
