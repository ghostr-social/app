import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets(
    'an explicit jump creates the farther decoder only after navigation',
    (tester) async {
      final fixture = FeedPreparationFixture(postCount: 4);
      addTearDown(fixture.updates.close);
      await fixture.pump(tester);
      fixture.publishWindow(1, 'p0', ['p3']);
      await fixture.settle(tester);

      expect(fixture.platform.creationsFor(fixture.url('p3')), 0);
      final element = tester.element(find.byType(PageView));
      element.read<FeedCubit>().pageChanged(3);
      await fixture.settle(tester);
      await fixture.settle(tester);

      final state = element.read<FeedCubit>().state as FeedLoaded;
      expect(state.activeIndex, 3);
      expect(fixture.platform.creationsFor(fixture.url('p3')), 1);
      final preparedPlayer = fixture.platform.playerFor(fixture.url('p3'));
      expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
      expect(
        fixture.platform.commands,
        isNot(contains('seek:$preparedPlayer:0')),
      );
      expect(fixture.platform.commands, contains('play:$preparedPlayer'));
    },
  );
}
