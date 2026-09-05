import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a farther prepared item cannot take the next decoder slot', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 5);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p2', 'p3']);
    await fixture.settle(tester);

    expect(fixture.platform.playerCount, 1);
    expect(fixture.platform.creationsFor(fixture.url('p2')), 0);
    expect(find.text('Caption p0'), findsOneWidget);
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}
