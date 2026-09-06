import 'dart:ui' show SemanticsAction;

import 'package:flutter_test/flutter_test.dart';

import '../support/feed_preparation_fixture.dart';
import '../support/feed_preview_gestures.dart';

void main() {
  testWidgets('memory pressure retires a preview and keeps current playback', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    addTearDown(fixture.updates.close);
    final semantics = tester.ensureSemantics();
    try {
      await fixture.pump(tester);
      fixture.publish(1, 'p0', 'p1');
      await fixture.settle(tester);
      await fixture.swipe(tester);
      fixture.publishWindow(2, 'p1', ['p0', 'p2']);
      await fixture.settle(tester);
      final current = fixture.platform.playerFor(fixture.url('p1'));
      final gesture = await tester.dragPreview(0.23);
      expect(fixture.platform.playerCount, 2);

      tester.binding.handleMemoryPressure();
      await tester.pumpPreview();
      expect(fixture.platform.playerCount, 1);
      expect(fixture.platform.isPlaying(current), isTrue);
      await gesture.cancel();
      await tester.pumpPreview();
      final forward = await tester.dragPreview(-0.23);
      expect(fixture.platform.playerCount, 1);
      expect(fixture.platform.creationsFor(fixture.url('p1')), 1);
      expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
      expect(fixture.platform.audibleOverlap, isFalse);
      expect(find.text('Caption p1').hitTestable(), findsOneWidget);
      expect(
        tester
            .getSemantics(find.byTooltip('Like video'))
            .getSemanticsData()
            .hasAction(SemanticsAction.tap),
        isTrue,
      );
      await forward.cancel();
    } finally {
      semantics.dispose();
    }
  });
}
