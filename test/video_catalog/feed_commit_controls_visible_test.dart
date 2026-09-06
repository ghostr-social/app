import 'dart:ui' show SemanticsAction;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';
import '../support/gated_feed_history.dart';

void main() {
  testWidgets(
    'committed post controls remain visible while a swipe awaits persistence',
    (tester) async {
      final semantics = tester.ensureSemantics();
      try {
        final history = GatedFeedHistory();
        addTearDown(history.unblock);
        final source = FakeVideoCatalogRepository(
          forYouFeed: [
            samplePost(id: 'first', caption: 'Visible video'),
            samplePost(id: 'second', caption: 'Uncommitted video'),
          ],
        );
        await tester.pumpWidget(
          feedScreenHarness(
            source,
            options: FeedScreenHarnessOptions(
              watch: FeedWatchDependencies(
                tracker: WatchHistoryTracker(
                  history: history,
                  failureReporter: RecordingFailureReporter(),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        await tester.drag(find.byType(Scrollable), const Offset(0, -600));
        await tester.pump();
        expect(
          history.pending.isCompleted,
          isTrue,
          reason: 'the swipe reached navigation persistence',
        );

        final like = find.byTooltip('Like video').hitTestable();
        expect(
          like,
          findsOneWidget,
          reason:
              'the committed controls must stay on screen until focus commits',
        );
        expect(
          tester
              .getSemantics(like)
              .getSemanticsData()
              .hasAction(SemanticsAction.tap),
          isTrue,
        );
        expect(find.text('Visible video').hitTestable(), findsOneWidget);
        expect(find.text('Uncommitted video'), findsNothing);

        await tester.tap(like);
        await tester.pumpAndSettle();
        expect(find.byTooltip('Unlike video').hitTestable(), findsOneWidget);
        expect(find.text('Uncommitted video'), findsNothing);

        history.unblock();
        await tester.pumpAndSettle();
        expect(find.text('Uncommitted video').hitTestable(), findsOneWidget);
      } finally {
        semantics.dispose();
      }
    },
  );
}
