import 'dart:ui' show SemanticsAction;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';
import '../support/feed_preview_gestures.dart';

void main() {
  testWidgets('canceling a backward preview restores the forward spare', (
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
      expect(fixture.platform.creationsFor(fixture.url('p0')), 2);

      await gesture.cancel();
      await tester.pumpPreview();

      final state =
          tester.element(find.byType(PageView)).read<FeedCubit>().state
              as FeedLoaded;
      expect(state.roster.active.id.value, 'p1');
      expect(tester.widget<PageView>(find.byType(PageView)).controller!.page, 1);
      expect(fixture.platform.creationsFor(fixture.url('p1')), 1);
      expect(fixture.platform.creationsFor(fixture.url('p2')), 2);
      expect(fixture.platform.isPlaying(current), isTrue);
      expect(fixture.platform.playerCount, 2);
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
    } finally {
      semantics.dispose();
    }
  });
}
