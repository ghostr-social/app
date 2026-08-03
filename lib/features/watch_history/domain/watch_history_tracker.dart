import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

class WatchHistoryTracker {
  const WatchHistoryTracker({
    required WatchHistoryRepository history,
    required AppSettingsRepository settings,
    required FailureReporter failureReporter,
    Clock clock = systemClock,
  })  : _history = history,
        _settings = settings,
        _failureReporter = failureReporter,
        _clock = clock;

  final WatchHistoryRepository _history;
  final AppSettingsRepository _settings;
  final FailureReporter _failureReporter;
  final Clock _clock;

  Future<void> videoWatched(VideoPost post) async {
    try {
      final settings = await _settings.load();
      if (!settings.hideWatchedVideos) return;
      await _history.snapshotForActiveAccount().record(
            WatchHistoryEntry.fromPost(post, _clock().toUtc()),
          );
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'WatchHistoryTracker.videoWatched',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
