import 'dart:ui' show SemanticsAction;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/caption_text.dart';

import '../support/feed_preparation_fixture.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('scrolling an expanded caption preserves prepared video focus', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    fixture.posts[1] = samplePost(
      id: 'p1',
      caption: List.generate(30, (index) => 'Caption line $index').join('\n'),
    ).withMedia(fixture.posts[1].media);
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
      final next = fixture.platform.playerFor(fixture.url('p2'));
      await tester.tap(find.byType(CaptionText));
      await tester.pumpAndSettle();
      final caption = find.ancestor(
        of: find.byType(CaptionText),
        matching: find.byType(SingleChildScrollView),
      );
      final gesture = await tester.startGesture(tester.getCenter(caption));
      try {
        await gesture.moveBy(const Offset(0, -60));
        await tester.pump();
        await gesture.moveBy(const Offset(0, -60));
        await fixture.settle(tester);
        final scrollable = tester.state<ScrollableState>(
          find.descendant(of: caption, matching: find.byType(Scrollable)),
        );
        expect(scrollable.position.pixels, greaterThan(0));
        expect(fixture.platform.disposed, isNot(contains(next)));
        expect(fixture.platform.creationsFor(fixture.url('p2')), 1);
        expect(fixture.platform.isPlaying(current), isTrue);
        final state = tester.element(caption).read<FeedCubit>().state;
        expect((state as FeedLoaded).roster.active.id.value, 'p1');
        expect(
          tester
              .getSemantics(find.byTooltip('Like video'))
              .getSemanticsData()
              .hasAction(SemanticsAction.tap),
          isTrue,
        );
      } finally {
        await gesture.up();
        await fixture.settle(tester);
      }
      expect(fixture.platform.disposed, isNot(contains(next)));
      expect(fixture.platform.creationsFor(fixture.url('p2')), 1);
      expect(fixture.platform.audibleOverlap, isFalse);
    } finally {
      semantics.dispose();
    }
  });
}
