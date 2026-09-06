import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets(
    'available preparation stays reachable for an explicit reverse swipe',
    (tester) async {
      final fixture = FeedPreparationFixture();
      addTearDown(fixture.updates.close);
      await fixture.pump(tester);
      fixture.publish(1, 'p0', 'p1');
      await fixture.settle(tester);
      await fixture.swipe(tester);
      fixture.publishWindow(2, 'p1', ['p0', 'p2']);
      await fixture.settle(tester);

      final cubit = tester.element(find.byType(PageView)).read<FeedCubit>();
      final loaded = cubit.state as FeedLoaded;
      expect(loaded.roster.active.id.value, 'p1');
      expect(find.text('Caption p1'), findsOneWidget);
      expect(
        loaded.preparation.forMedia(fixture.posts[0].media),
        isNotNull,
        reason:
            'A ready previous presentation must remain available for reversal.',
      );
      expect(fixture.platform.peakPlayerCount, lessThanOrEqualTo(2));
    },
  );
}
