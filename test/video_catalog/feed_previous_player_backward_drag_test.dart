import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a backward drag reveals the warm previous video', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publish(1, 'p0', 'p1');
    await fixture.settle(tester);
    final previousPlayer = fixture.platform.playerFor(fixture.url('p0'));

    final page = find.byType(PageView);
    final height = tester.getSize(page).height;
    final forward = await tester.startGesture(tester.getCenter(page));
    await forward.moveBy(Offset(0, -height * 0.23));
    await tester.pump(const Duration(milliseconds: 16));
    await forward.up();
    await tester.pump(const Duration(milliseconds: 500));
    final state = tester.element(page).read<FeedCubit>().state as FeedLoaded;
    expect(state.roster.active.id.value, 'p1');
    final currentPlayer = fixture.platform.playerFor(fixture.url('p1'));

    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, height * 0.23));
    await tester.pump(const Duration(milliseconds: 16));

    expect(_feedCard('p0'), findsOneWidget);
    final previousTexture = _textureFor('p0', previousPlayer);
    expect(previousTexture, findsOneWidget);
    expect(
      tester.getRect(previousTexture).overlaps(tester.getRect(page)),
      true,
    );
    expect(fixture.platform.disposed, isNot(contains(previousPlayer)));
    expect(fixture.platform.creationsFor(fixture.url('p0')), 1);
    expect(fixture.platform.isPlaying(previousPlayer), isFalse);
    expect(fixture.platform.isPlaying(currentPlayer), isTrue);
    expect(fixture.platform.audibleOverlap, isFalse);
    expect(
      (tester.element(page).read<FeedCubit>().state as FeedLoaded)
          .roster
          .active
          .id
          .value,
      'p1',
    );
    await gesture.cancel();
  });
}

Finder _textureFor(String id, int textureId) {
  return find.descendant(
    of: _feedCard(id),
    matching: find.byWidgetPredicate(
      (widget) => widget is Texture && widget.textureId == textureId,
      skipOffstage: false,
    ),
  );
}

Finder _feedCard(String id) {
  return find.byWidgetPredicate(
    (widget) => widget is FeedCard && widget.post.id.value == id,
    skipOffstage: false,
  );
}
