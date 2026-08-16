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
  testWidgets('a swiped video stays active while its watch commits', (
    tester,
  ) async {
    final history = _SecondWatchGatedHistory();
    final playback = FakeVideoPlaybackPort();
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
          playbackPort: playback,
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
    expect(find.text('Uncommitted video'), findsOneWidget);
    final cubit = tester.element(find.byType(FeedScreen)).read<FeedCubit>();
    expect((cubit.state as FeedLoaded).activeIndex, 1);
    expect(_latestActivity(playback, 'e:first'), isFalse);
    expect(_latestActivity(playback, 'e:second'), isTrue);

    history.release.complete();
    await tester.pumpAndSettle();
    expect(find.text('Uncommitted video'), findsOneWidget);
  });
}

bool _latestActivity(FakeVideoPlaybackPort playback, String videoId) {
  return playback.requests
      .lastWhere((request) => request.videoId?.value == videoId)
      .isActive;
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
