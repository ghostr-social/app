import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('visible roster updates retain a pending page watch', () async {
    final history = _SecondWatchGatedHistory();
    final first = samplePost(id: 'first');
    final second = samplePost(id: 'second');
    final viewer = FeedViewer(
      watchTracker: WatchHistoryTracker(
        history: history,
        failureReporter: RecordingFailureReporter(),
      ),
    );
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      await viewer.dispose();
    });
    viewer.landedOn([first, second], 0);
    await pumpEventQueue();

    final pending = viewer.prepareToShow(second) as Future<bool>;
    await history.secondStarted.future;
    viewer.rosterChanged([first, second], 0);
    history.release.complete();

    expect(await pending, isTrue);
    expect(viewer.prepareToShow(second), isTrue);
    expect(history.writes, 2);
  });
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (++writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
