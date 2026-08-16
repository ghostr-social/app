import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('closing comments restores the same current feed video', (
    tester,
  ) async {
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

    await tester.tap(find.byTooltip('Open comments'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 500));
    await tester.tapAt(const Offset(10, 10));
    await tester.pump();

    await tester.pumpAndSettle();
    expect(find.text('First watched video'), findsOneWidget);
    expect(find.text('Second unseen video'), findsNothing);
  });
}
