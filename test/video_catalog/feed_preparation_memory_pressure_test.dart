import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('memory pressure releases the deep prepared reserve', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 7);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3', 'p4', 'p5']);
    await fixture.settle(tester);
    expect(fixture.platform.playerCount, 6);

    tester.binding.handleMemoryPressure();
    await fixture.settle(tester);

    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.creationsFor(fixture.url('p1')), 1);
    for (final id in ['p2', 'p3', 'p4', 'p5']) {
      expect(
        fixture.platform.disposed,
        contains(fixture.platform.playerFor(fixture.url(id))),
      );
    }
  });
}
