import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
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
  test('reports a failed watch recording instead of throwing', () async {
    final reporter = RecordingFailureReporter();
    final tracker = WatchHistoryTracker(
      history: _RejectingWatchHistoryRepository(),
      settings: FakeAppSettingsRepository(AppSettings.defaults()),
      failureReporter: reporter,
    );

    await tracker.videoWatched(samplePost());

    expect(reporter.sources, ['WatchHistoryTracker.videoWatched']);
  });
}
