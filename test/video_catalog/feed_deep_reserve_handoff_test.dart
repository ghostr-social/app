import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('a promoted deep reserve reuses its prepared player', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 4);
    addTearDown(fixture.updates.close);
    await fixture.pump(tester);
    fixture.publishWindow(1, 'p0', ['p3']);
    await fixture.settle(tester);

    expect(fixture.platform.creationsFor(fixture.url('p3')), 1);
    final preparedPlayer = fixture.platform.playerFor(fixture.url('p3'));
    final element = tester.element(find.byType(PageView));
    element.read<FeedCubit>().pageChanged(3);
    await fixture.settle(tester);

    final state = element.read<FeedCubit>().state as FeedLoaded;
    expect(state.activeIndex, 3);
    expect(fixture.platform.creationsFor(fixture.url('p3')), 1);
    expect(fixture.platform.disposed, isNot(contains(preparedPlayer)));
    expect(fixture.platform.commands, contains('play:$preparedPlayer'));
  });
}
