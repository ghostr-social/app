import 'dart:ui' show SemanticsAction;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('For You retains five viewed videos for an explicit back swipe', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    try {
      final repository = FakeVideoCatalogRepository(
        forYouFeed: List.generate(
          8,
          (index) => samplePost(id: 'post-$index', caption: 'Clip $index'),
        ),
      );
      final tracker = WatchHistoryTracker(
        history: FakeWatchHistoryRepository(),
        failureReporter: RecordingFailureReporter(),
      );
      final focus = FakeFeedFocusPort();
      await tester.pumpWidget(
        feedScreenHarness(
          repository,
          options: FeedScreenHarnessOptions(
            watch: FeedWatchDependencies(tracker: tracker),
            focus: focus,
          ),
        ),
      );
      await tester.pumpAndSettle();

      for (var swipe = 0; swipe < 6; swipe += 1) {
        final page = find.byType(PageView);
        await tester.drag(page, Offset(0, -tester.getSize(page).height * 0.8));
        await tester.pumpAndSettle();
      }
      final context = tester.element(find.byType(PageView));
      var loaded = context.read<FeedCubit>().state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), [
        'post-1',
        'post-2',
        'post-3',
        'post-4',
        'post-5',
        'post-6',
        'post-7',
      ]);
      expect(loaded.activeIndex, 5);
      expect(focus.focuses.last.currentIndex, 5);
      expect(focus.focuses.last.window.map((post) => post.id.value), [
        'post-1',
        'post-2',
        'post-3',
        'post-4',
        'post-5',
        'post-6',
        'post-7',
      ]);

      for (var swipe = 0; swipe < 5; swipe += 1) {
        final page = find.byType(PageView);
        await tester.drag(page, Offset(0, tester.getSize(page).height * 0.8));
        await tester.pumpAndSettle();
      }
      loaded = context.read<FeedCubit>().state as FeedLoaded;
      expect(loaded.posts[loaded.activeIndex].id.value, 'post-1');
      expect(find.text('Clip 1').hitTestable(), findsOneWidget);
      expect(find.byTooltip('Like video').hitTestable(), findsOneWidget);
      final page = find.byType(PageView);
      await tester.drag(page, Offset(0, tester.getSize(page).height * 0.8));
      await tester.pumpAndSettle();
      expect(find.text('Clip 1').hitTestable(), findsOneWidget);
      expect(find.text('Clip 0'), findsNothing);
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
