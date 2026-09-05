import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a retiring player blocks the next decoder until disposal', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 3);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2']);
    await fixture.settle(tester);
    final retiring = fixture.platform.playerFor(fixture.url('p0'));
    expect(fixture.platform.playerCount, 2);

    fixture.platform.blockDisposal();
    await _swipe(tester);
    fixture.publishWindow(2, 'p1', ['p2']);
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.creationsFor(fixture.url('p2')), 0);
    expect(fixture.platform.peakPlayerCount, 2);

    fixture.platform.releaseDisposal();
    await tester.pump();
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await fixture.settle(tester);
    expect(fixture.platform.disposed, contains(retiring));
    expect(fixture.platform.creationsFor(fixture.url('p2')), 1);
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.peakPlayerCount, 2);
  });
}

Future<void> _swipe(WidgetTester tester) async {
  final page = find.byType(PageView);
  final gesture = await tester.startGesture(tester.getCenter(page));
  await gesture.moveBy(Offset(0, -tester.getSize(page).height * 0.23));
  await tester.pump(const Duration(milliseconds: 16));
  await gesture.up();
  await tester.pump(const Duration(milliseconds: 500));
}
