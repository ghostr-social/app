import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'returning after history clear records the visible video again',
    () async {
      final history = FakeWatchHistoryRepository();
      final viewer = FeedViewer(
        watchTracker: WatchHistoryTracker(
          history: history,
          failureReporter: RecordingFailureReporter(),
        ),
      );
      final post = samplePost(id: 'visible');
      viewer.landedOn([post], 0);
      await pumpEventQueue();
      expect(history.entries, hasLength(1));

      viewer.visibilityChanged(false);
      await history.clear();
      viewer.visibilityChanged(true);
      viewer.rosterChanged([post], 0);
      await pumpEventQueue();

      expect(history.entries.map((entry) => entry.videoId), ['e:visible']);
    },
  );
}
