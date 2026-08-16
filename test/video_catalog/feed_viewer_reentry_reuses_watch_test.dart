import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'temporary reentry does not rewrite a committed session video',
    () async {
      final history = _CountingHistory();
      final viewer = FeedViewer(
        watchTracker: WatchHistoryTracker(
          history: history,
          failureReporter: RecordingFailureReporter(),
        ),
      );
      final post = samplePost(id: 'visible');
      viewer.landedOn([post], 0);
      await pumpEventQueue();
      expect(history.writes, 1);

      viewer.visibilityChanged(false);
      viewer.visibilityChanged(true);
      viewer.rosterChanged([post], 0);
      await pumpEventQueue();

      expect(history.writes, 1);
    },
  );
}

final class _CountingHistory extends FakeWatchHistoryRepository {
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) {
    writes += 1;
    return super.record(entry);
  }
}
