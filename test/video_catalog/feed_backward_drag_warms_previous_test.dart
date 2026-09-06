import 'dart:ui' show SemanticsAction;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a reverse drag warms the previous video within two decoders', (
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
      final next = fixture.platform.playerFor(fixture.url('p2'));
      final page = find.byType(PageView);
      final gesture = await tester.startGesture(tester.getCenter(page));
      await gesture.moveBy(Offset(0, tester.getSize(page).height * 0.23));
      for (var frame = 0; frame < 8; frame++) {
        await tester.pump(const Duration(milliseconds: 20));
        await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      }

      expect(
        fixture.platform.creationsFor(fixture.url('p0')),
        2,
        reason:
            'the visible previous page needs a prepared decoder before release',
      );
      final previous = fixture.platform.sources.entries
          .where((entry) => entry.value.uri == fixture.url('p0'))
          .last
          .key;
      final texture = find.byWidgetPredicate(
        (widget) => widget is Texture && widget.textureId == previous,
      );
      expect(texture, findsOneWidget);
      expect(
        tester.getRect(texture).overlaps(tester.getRect(page)),
        isTrue,
        reason: 'the prepared previous video stays visible under the finger',
      );
      expect(fixture.platform.disposed, contains(next));
      expect(fixture.platform.isPlaying(current), isTrue);
      expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
      expect(fixture.platform.audibleOverlap, isFalse);
      expect(
        (tester.element(page).read<FeedCubit>().state as FeedLoaded)
            .roster
            .active
            .id
            .value,
        'p1',
      );
      expect(find.text('Caption p1').hitTestable(), findsOneWidget);
      expect(
        tester
            .getSemantics(find.byTooltip('Like video'))
            .getSemanticsData()
            .hasAction(SemanticsAction.tap),
        isTrue,
      );
      await gesture.cancel();
    } finally {
      semantics.dispose();
    }
  });
}
