import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('resuming an ordinary feed cannot render its exited video', (
    tester,
  ) async {
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    addTearDown(
      () => tester.binding.handleAppLifecycleStateChanged(
        AppLifecycleState.resumed,
      ),
    );
    final history = FakeWatchHistoryRepository();
    final source = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'first', caption: 'First watched video'),
        samplePost(id: 'second', caption: 'Second unseen video'),
      ],
    );
    final reporter = RecordingFailureReporter();
    final feed = WatchAwareVideoFeedRepository(
      feed: source,
      history: history,
      failureReporter: reporter,
    );
    await tester.pumpWidget(
      feedScreenHarness(
        source,
        options: FeedScreenHarnessOptions(
          feed: feed,
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: reporter,
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('First watched video'), findsOneWidget);

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(find.text('First watched video'), findsNothing);
    await tester.pumpAndSettle();

    expect(find.text('First watched video'), findsNothing);
    expect(find.text('Second unseen video'), findsOneWidget);
  });
}
