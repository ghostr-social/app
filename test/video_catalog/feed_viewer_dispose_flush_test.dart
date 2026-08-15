import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('disposing the viewer flushes an in-flight watch record', () async {
    final history = _GatedWatchHistoryRepository();
    final viewer = FeedViewer(
      watchTracker: WatchHistoryTracker(
        history: history,
        failureReporter: RecordingFailureReporter(),
      ),
    );
    viewer.landedOn([samplePost()], 0);
    await history.started.future;
    var finished = false;

    final disposal = viewer.dispose()..then((_) => finished = true);
    await pumpEventQueue();
    expect(finished, isFalse);
    history.release.complete();
    await disposal;

    expect(finished, isTrue);
  });
}

final class _GatedWatchHistoryRepository extends FakeWatchHistoryRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    started.complete();
    await release.future;
    await super.record(entry);
  }
}
