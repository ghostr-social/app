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
  testWidgets(
    'For You retains three viewed videos for an explicit back swipe',
    (tester) async {
      final repository = FakeVideoCatalogRepository(
        forYouFeed: List.generate(6, (index) => samplePost(id: 'post-$index')),
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

      for (var swipe = 0; swipe < 4; swipe += 1) {
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
      ]);
      expect(loaded.activeIndex, 3);
      expect(focus.focuses.last.currentIndex, 3);
      expect(focus.focuses.last.window.map((post) => post.id.value), [
        'post-1',
        'post-2',
        'post-3',
        'post-4',
        'post-5',
      ]);

      for (var swipe = 0; swipe < 3; swipe += 1) {
        final page = find.byType(PageView);
        await tester.drag(page, Offset(0, tester.getSize(page).height * 0.8));
        await tester.pumpAndSettle();
      }
      loaded = context.read<FeedCubit>().state as FeedLoaded;
      expect(loaded.posts[loaded.activeIndex].id.value, 'post-1');
    },
  );
}
