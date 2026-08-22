import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a retiring player blocks a seventh prepared surface', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 8);
    addTearDown(fixture.updates.close);
    addTearDown(fixture.platform.releaseDisposal);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3', 'p4', 'p5']);
    await fixture.settle(tester);
    fixture.platform.blockDisposal();

    await fixture.swipe(tester);
    fixture.publishWindow(2, 'p1', ['p2', 'p3', 'p4', 'p5', 'p6']);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.platform.playerCount, 6);
    expect(fixture.platform.creationsFor(fixture.url('p6')), 0);

    fixture.platform.releaseDisposal();
    await fixture.settle(tester);
    expect(fixture.platform.playerCount, 6);
    expect(fixture.platform.creationsFor(fixture.url('p6')), 1);
    expect(fixture.platform.peakPlayerCount, 6);
  });
}
