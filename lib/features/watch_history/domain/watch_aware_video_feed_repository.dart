import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

class WatchAwareVideoFeedRepository implements VideoFeedRepository {
  const WatchAwareVideoFeedRepository({
    required VideoFeedRepository feed,
    required WatchHistoryRepository history,
    required AppSettingsRepository settings,
    required FailureReporter failureReporter,
  })  : _feed = feed,
        _history = history,
        _settings = settings,
        _failureReporter = failureReporter;

  final VideoFeedRepository _feed;
  final WatchHistoryRepository _history;
  final AppSettingsRepository _settings;
  final FailureReporter _failureReporter;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final posts = await _feed.loadFeed(kind);
    if (!excludeWatched || !await _isEnabled()) return posts;
    final watched = await _watchedTimes();
    if (watched.isEmpty) return posts;
    final fresh =
        posts.where((post) => !watched.containsKey(_coordinate(post))).toList();
    if (fresh.isNotEmpty) return List<VideoPost>.unmodifiable(fresh);
    return _leastRecentlyWatched(posts, watched);
  }

  // Only reached when every fetched video is already watched: the feed must
  // not dead-end, so repeat the pool starting from the videos watched
  // longest ago.
  List<VideoPost> _leastRecentlyWatched(
    List<VideoPost> posts,
    Map<String, DateTime> watched,
  ) {
    final ordered = posts.toList()
      ..sort((left, right) {
        return watched[_coordinate(left)]!.compareTo(
          watched[_coordinate(right)]!,
        );
      });
    return List<VideoPost>.unmodifiable(ordered);
  }

  String _coordinate(VideoPost post) {
    return VideoInteractionTarget.fromPost(post).value;
  }

  Future<bool> _isEnabled() async {
    try {
      return (await _settings.load()).hideWatchedVideos;
    } on Object catch (error, stackTrace) {
      _report('WatchAwareVideoFeedRepository.settings', error, stackTrace);
      return false;
    }
  }

  Future<Map<String, DateTime>> _watchedTimes() async {
    try {
      final entries = await _history.snapshotForActiveAccount().load();
      return {for (final entry in entries) entry.videoId: entry.watchedAt};
    } on Object catch (error, stackTrace) {
      _report('WatchAwareVideoFeedRepository.history', error, stackTrace);
      return const <String, DateTime>{};
    }
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: source,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
