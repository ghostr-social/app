import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('only the two immediate previous players stay warm', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 9);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3', 'p4', 'p5']);
    await fixture.settle(tester);
    final first = fixture.platform.playerFor(fixture.url('p0'));

    await fixture.swipe(tester);
    fixture.publishWindow(2, 'p1', ['p2', 'p3', 'p4', 'p5', 'p6']);
    await fixture.settle(tester);
    final older = fixture.platform.playerFor(fixture.url('p1'));

    await fixture.swipe(tester);
    fixture.publishWindow(3, 'p2', ['p3', 'p4', 'p5', 'p6', 'p7']);
    await fixture.settle(tester);
    final previous = fixture.platform.playerFor(fixture.url('p2'));

    await fixture.swipe(tester);
    fixture.publishWindow(4, 'p3', ['p4', 'p5', 'p6', 'p7', 'p8']);
    await fixture.settle(tester);
    final current = fixture.platform.playerFor(fixture.url('p3'));

    expect(fixture.platform.disposed, contains(first));
    expect(fixture.platform.disposed, isNot(contains(older)));
    expect(fixture.platform.disposed, isNot(contains(previous)));
    expect(fixture.platform.creationsFor(fixture.url('p0')), 1);
    expect(fixture.platform.creationsFor(fixture.url('p1')), 1);
    expect(fixture.platform.isPlaying(older), isFalse);
    expect(fixture.platform.isPlaying(previous), isFalse);
    expect(fixture.platform.isPlaying(current), isTrue);
    expect(fixture.platform.playerCount, 8);
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}
