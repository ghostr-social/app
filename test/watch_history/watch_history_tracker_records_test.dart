import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'records the watched post with its coordinate and the clock time',
    () async {
      final history = FakeWatchHistoryRepository();
      final watchedAt = DateTime.utc(2026, 3, 12, 10, 30);
      final tracker = WatchHistoryTracker(
        history: history,
        failureReporter: RecordingFailureReporter(),
        clock: () => watchedAt,
      );

      await tracker.videoWatched(samplePost(id: 'post-1'));

      final entry = history.entries.single;
      expect(entry.videoId, 'e:post-1');
      expect(entry.title, 'A relay-side banger');
      expect(entry.creatorName, 'Nora Relay');
      expect(entry.watchedAt, watchedAt);
      expect(entry.watchedAt.isUtc, isTrue);
    },
  );
}
