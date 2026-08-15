import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

class _RejectingWatchHistoryRepository extends FakeWatchHistoryRepository {
  @override
  Future<void> record(WatchHistoryEntry entry) async {
    throw const AppFailure('Could not save watch history.');
  }
}

void main() {
  test('reports and exposes a failed watch recording', () async {
    final reporter = RecordingFailureReporter();
    final tracker = WatchHistoryTracker(
      history: _RejectingWatchHistoryRepository(),
      failureReporter: reporter,
    );

    await expectLater(
      tracker.videoWatched(samplePost()),
      throwsA(isA<AppFailure>()),
    );

    expect(reporter.sources, ['WatchHistoryTracker.videoWatched']);
  });
}
