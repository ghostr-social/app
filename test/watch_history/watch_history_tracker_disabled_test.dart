import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('records nothing when hiding watched videos is disabled', () async {
    final history = FakeWatchHistoryRepository();
    final tracker = WatchHistoryTracker(
      history: history,
      settings: FakeAppSettingsRepository(
        AppSettings.defaults().copyWith(hideWatchedVideos: false),
      ),
      failureReporter: RecordingFailureReporter(),
    );

    await tracker.videoWatched(samplePost());

    expect(history.entries, isEmpty);
  });
}
