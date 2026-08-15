import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('an uncommitted adjacent video is never rendered', (
    tester,
  ) async {
    final history = _SecondWatchGatedHistory();
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

    expect(find.text('Visible video'), findsOneWidget);
    expect(find.text('Uncommitted video'), findsNothing);
    final page = find.byType(Scrollable);
    await tester.drag(page, const Offset(0, -600));
    await history.secondStarted.future;
    await tester.pump();
    expect(find.text('Uncommitted video'), findsNothing);

    history.release.complete();
    await tester.pumpAndSettle();
    expect(find.text('Uncommitted video'), findsOneWidget);
  });
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes += 1;
    if (writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
