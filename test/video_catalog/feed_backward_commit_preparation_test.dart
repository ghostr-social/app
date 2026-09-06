import 'dart:ui' show SemanticsAction;

import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';
import '../support/feed_preview_gestures.dart';

void main() {
  testWidgets(
    'a backward commit reuses its preview and prepares further back',
    (tester) async {
      final fixture = FeedPreparationFixture(postCount: 4);
      addTearDown(fixture.updates.close);
      final semantics = tester.ensureSemantics();
      try {
        await fixture.pump(tester);
        fixture.publish(1, 'p0', 'p1');
        await fixture.settle(tester);
        await fixture.swipe(tester);
        fixture.publishWindow(2, 'p1', ['p0', 'p2']);
        await fixture.settle(tester);
        await fixture.swipe(tester);
        fixture.publishWindow(3, 'p2', ['p0', 'p1', 'p3']);
        await fixture.settle(tester);

        final gesture = await tester.dragPreview(0.23);
        expect(fixture.platform.creationsFor(fixture.url('p1')), 2);
        await gesture.up();
        await fixture.settle(tester);
        fixture.publishWindow(4, 'p1', ['p0', 'p2', 'p3']);
        await fixture.settle(tester);

        expect(fixture.platform.creationsFor(fixture.url('p1')), 2);
        expect(fixture.platform.creationsFor(fixture.url('p0')), 2);
        expect(fixture.platform.playerCount, 2);
        expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
        expect(fixture.platform.audibleOverlap, isFalse);
        final live = fixture.platform.sources.entries.where(
          (entry) => !fixture.platform.disposed.contains(entry.key),
        );
        expect(
          live.map((entry) => entry.value.uri),
          unorderedEquals([fixture.url('p0'), fixture.url('p1')]),
        );
        final current = live.singleWhere(
          (entry) => entry.value.uri == fixture.url('p1'),
        );
        expect(fixture.platform.isPlaying(current.key), isTrue);
        expect(find.text('Caption p1').hitTestable(), findsOneWidget);
        expect(
          tester
              .getSemantics(find.byTooltip('Like video'))
              .getSemanticsData()
              .hasAction(SemanticsAction.tap),
          isTrue,
        );
        final previous = await tester.dragPreview(0.23);
        await previous.up();
        await fixture.settle(tester);
        expect(find.text('Caption p0').hitTestable(), findsOneWidget);
        expect(fixture.platform.playerCount, 1);
        expect(fixture.platform.creationsFor(fixture.url('p0')), 2);

        fixture.publish(5, 'p0', 'p1');
        await fixture.settle(tester);
        final forward = await tester.dragPreview(-0.23);
        expect(fixture.platform.playerCount, 2);
        await forward.up();
        await fixture.settle(tester);
        expect(find.text('Caption p1').hitTestable(), findsOneWidget);
        expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
        expect(fixture.platform.audibleOverlap, isFalse);
      } finally {
        semantics.dispose();
      }
    },
  );
}
