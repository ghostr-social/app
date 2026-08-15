import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

class WatchHistoryTracker {
  const WatchHistoryTracker({
    required WatchHistoryRepository history,
    required FailureReporter failureReporter,
    Clock clock = systemClock,
  }) : _history = history,
       _failureReporter = failureReporter,
       _clock = clock;

  final WatchHistoryRepository _history;
  final FailureReporter _failureReporter;
  final Clock _clock;

  Future<void> videoWatched(VideoPost post) async {
    try {
      await _history.snapshotForActiveAccount().record(
        WatchHistoryEntry.fromPost(post, _clock().toUtc()),
      );
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'WatchHistoryTracker.videoWatched',
        error: error,
        stackTrace: stackTrace,
      );
      rethrow;
    }
  }
}
