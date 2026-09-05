import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('core warms only the active and immediate next player', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 9);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p1', 'p2', 'p3', 'p4', 'p5']);
    await fixture.settle(tester);
    final previous = fixture.platform.playerFor(fixture.url('p0'));

    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.creationsFor(fixture.url('p2')), 0);
    expect(find.text('Caption p0'), findsOneWidget);
    await fixture.swipe(tester);
    final state =
        tester.element(find.byType(PageView)).read<FeedCubit>().state
            as FeedLoaded;
    expect(state.posts[state.activeIndex].id.value, 'p1');
    fixture.publishWindow(2, 'p1', ['p2', 'p3', 'p4', 'p5', 'p6']);
    await fixture.settle(tester);
    expect(find.text('Caption p1'), findsOneWidget);

    expect(fixture.platform.playerCount, 2);
    expect(fixture.platform.disposed, contains(previous));
    expect(fixture.platform.creationsFor(fixture.url('p2')), 1);
    expect(fixture.platform.creationsFor(fixture.url('p3')), 0);
    expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
    expect(fixture.platform.audibleOverlap, isFalse);
    expect(find.text('Caption p1'), findsOneWidget);
    tester.binding.handleMemoryPressure();
    await fixture.settle(tester);
    expect(fixture.platform.playerCount, 1);
    expect(find.text('Caption p1'), findsOneWidget);
  });
}
