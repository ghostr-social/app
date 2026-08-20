import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('the exact adjacent player is prepared and reused on swipe', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publish(1, 'p0', 'p1');
    await fixture.settle(tester);

    final p0 = fixture.platform.playerFor(fixture.url('p0'));
    final p1 = fixture.platform.playerFor(fixture.url('p1'));
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.commands, containsAll(['play:$p0', 'pause:$p1']));
    expect(find.text('Caption p0'), findsOneWidget);
    expect(find.text('Caption p1'), findsNothing);

    fixture.platform.commands.clear();
    await fixture.swipe(tester);
    final state = tester.element(find.byType(PageView)).read<FeedCubit>().state;
    expect((state as FeedLoaded).posts.map((post) => post.id.value), [
      'p1',
      'p2',
    ]);
    expect(state.activeIndex, 0);
    expect(fixture.platform.commands, contains('play:$p1'));
    expect(fixture.platform.creationsFor(fixture.url('p1')), 1);
    expect(find.text('Caption p0', skipOffstage: false), findsNothing);
    expect(find.byKey(const ValueKey('p0'), skipOffstage: false), findsNothing);
    expect(fixture.platform.playerCount, 1);
    expect(fixture.platform.disposed, contains(p0));
    expect(find.text('Caption p1'), findsOneWidget);

    fixture.publish(2, 'p1', 'p2');
    await fixture.settle(tester);
    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.creationsFor(fixture.url('p1')), 1);
    expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}
