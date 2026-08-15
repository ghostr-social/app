import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('records every visible video without a settings off-switch', () async {
    final history = FakeWatchHistoryRepository();
    final tracker = WatchHistoryTracker(
      history: history,
      failureReporter: RecordingFailureReporter(),
    );

    await tracker.videoWatched(samplePost());

    expect(history.entries.map((entry) => entry.videoId), ['e:post-1']);
  });
}
