import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('a passive refresh preserves an in-flight page transition', (
    tester,
  ) async {
    final history = _GatedHistory();
    final first = samplePost(id: 'first', caption: 'Active video');
    final source = FakeVideoCatalogRepository(
      forYouFeed: [
        first,
        samplePost(id: 'second', caption: 'Requested video'),
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
    final context = tester.element(find.byType(FeedScreen));
    final cubit = context.read<FeedCubit>();

    await tester.drag(find.byType(Scrollable), const Offset(0, -600));
    await history.secondStarted.future;
    final refresh = cubit.refresh();
    await tester.pump();

    history.release.complete();
    await refresh;
    await tester.pumpAndSettle();

    expect(find.text('Requested video').hitTestable(), findsOneWidget);
  });
}

final class _GatedHistory extends FakeWatchHistoryRepository {
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
