import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('an ordinary feed can swipe back to its previous video', (
    tester,
  ) async {
    final history = _CountingHistory();
    final source = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'first', caption: 'First video'),
        samplePost(id: 'second', caption: 'Second video'),
        samplePost(id: 'third', caption: 'Third video'),
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
    final pages = find.byType(PageView);
    final height = tester.getSize(pages).height;

    await tester.drag(pages, Offset(0, -height * 0.7));
    await tester.pumpAndSettle();
    expect(find.text('Second video').hitTestable(), findsOneWidget);

    await tester.drag(find.byType(PageView), Offset(0, height * 0.7));
    await tester.pumpAndSettle();
    expect(find.text('First video').hitTestable(), findsOneWidget);
    expect(history.writes, 2);
  });
}

final class _CountingHistory extends FakeWatchHistoryRepository {
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) {
    writes += 1;
    return super.record(entry);
  }
}
