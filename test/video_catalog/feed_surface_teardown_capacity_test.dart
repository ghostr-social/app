import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a retiring eighth player blocks the ninth until disposal', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 9);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3', 'p4', 'p5']);
    await fixture.settle(tester);
    final retiring = fixture.platform.playerFor(fixture.url('p0'));
    await fixture.swipe(tester);
    fixture.publishWindow(2, 'p1', ['p2', 'p3', 'p4', 'p5', 'p6']);
    await fixture.settle(tester);
    await fixture.swipe(tester);
    fixture.publishWindow(3, 'p2', ['p3', 'p4', 'p5', 'p6', 'p7']);
    await fixture.settle(tester);
    expect(fixture.platform.playerCount, 8);

    fixture.platform.blockDisposal();
    await _swipe(tester);
    fixture.publishWindow(4, 'p3', ['p4', 'p5', 'p6', 'p7', 'p8']);
    await tester.pump(const Duration(milliseconds: 100));

    expect(fixture.platform.playerCount, 8);
    expect(fixture.platform.creationsFor(fixture.url('p8')), 0);
    expect(fixture.platform.peakPlayerCount, 8);

    fixture.platform.releaseDisposal();
    await tester.pump();
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump(const Duration(milliseconds: 500));
    expect(fixture.platform.disposed, contains(retiring));
    expect(fixture.platform.creationsFor(fixture.url('p8')), 1);
    expect(fixture.platform.playerCount, 8);
    expect(fixture.platform.peakPlayerCount, 8);
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
