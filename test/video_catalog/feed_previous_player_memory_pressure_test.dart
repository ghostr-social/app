import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('memory pressure releases both warm previous players', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 5);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3']);
    await fixture.settle(tester);
    final older = fixture.platform.playerFor(fixture.url('p0'));

    await fixture.swipe(tester);
    fixture.publishWindow(2, 'p1', ['p2', 'p3']);
    await fixture.settle(tester);
    final previous = fixture.platform.playerFor(fixture.url('p1'));
    await fixture.swipe(tester);
    fixture.publishWindow(3, 'p2', ['p3']);
    await fixture.settle(tester);
    expect(fixture.platform.disposed, isNot(contains(older)));
    expect(fixture.platform.disposed, isNot(contains(previous)));

    tester.binding.handleMemoryPressure();
    await fixture.settle(tester);

    expect(fixture.platform.disposed, containsAll([older, previous]));
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}
