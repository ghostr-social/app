import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

final class WatchAwareVideoSearchRepository
    implements VideoSearchRepository, VideoSearchUpdates {
  const WatchAwareVideoSearchRepository({
    required VideoSearchRepository search,
    required VideoSearchUpdates updates,
    required WatchHistoryRepository history,
    required FailureReporter failureReporter,
  }) : _search = search,
       _updates = updates,
       _history = history,
       _failureReporter = failureReporter;

  final VideoSearchRepository _search;
  final VideoSearchUpdates _updates;
  final WatchHistoryRepository _history;
  final FailureReporter _failureReporter;
  static const _maxPageDigs = 3;

  @override
  Future<VideoFeedPage> searchVideos(
    String query, {
    DateTime? olderThan,
  }) async {
    final history = _historySnapshot();
    final page = await _search.searchVideos(query, olderThan: olderThan);
    return _dig(query, page, history);
  }

  @override
  Future<VideoFeedPage> loadMoreVideos(String query) async {
    final history = _historySnapshot();
    return _dig(query, await _search.loadMoreVideos(query), history);
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) {
    return _search.searchCreators(query);
  }

  @override
  Stream<VideoSearchSnapshot> watchVideos(String query) {
    return _updates
        .watchVideos(query)
        .asyncMap((snapshot) => _freshSnapshot(query, snapshot));
  }

  Future<VideoSearchSnapshot> _freshSnapshot(
    String query,
    VideoSearchSnapshot snapshot,
  ) async {
    final page = await _dig(query, snapshot.page, _historySnapshot());
    return VideoSearchSnapshot(
      revision: snapshot.revision,
      phase: snapshot.phase,
      page: page,
    );
  }

  Future<VideoFeedPage> _dig(
    String query,
    VideoFeedPage initial,
    WatchHistoryRepository history,
  ) async {
    var page = initial;
    for (var dig = 0; dig < _maxPageDigs; dig += 1) {
      final fresh = await _freshPage(page, history);
      if (fresh.posts.isNotEmpty || !page.hasMore) return fresh;
      page = await _search.loadMoreVideos(query);
    }
    return _freshPage(page, history);
  }

  Future<VideoFeedPage> _freshPage(
    VideoFeedPage page,
    WatchHistoryRepository history,
  ) async {
    try {
      return VideoFeedPage(
        posts: await history.filterUnwatched(page.posts),
        nextOlderThan: page.nextOlderThan,
      );
    } on Object catch (error, stackTrace) {
      _report(error, stackTrace);
      rethrow;
    }
  }

  WatchHistoryRepository _historySnapshot() {
    try {
      return _history.snapshotForActiveAccount();
    } on Object catch (error, stackTrace) {
      _report(error, stackTrace);
      rethrow;
    }
  }

  void _report(Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: 'WatchAwareVideoSearchRepository.history',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
