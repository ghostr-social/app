import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watched_video_index.dart';

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

  static const _maxPageDigs = 3;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final posts = await _feed.loadFeed(kind);
    if (!excludeWatched || !await _isEnabled()) return posts;
    final watched = await _watchedIndex();
    if (watched.isEmpty) return posts;
    final fresh = posts.where((post) => !watched.contains(post)).toList();
    if (fresh.isNotEmpty) return List<VideoPost>.unmodifiable(fresh);
    return _leastRecentlyWatched(posts, watched);
  }

  // Fully-watched pages are skipped by digging further into the past; unlike
  // the first load there is no replay fallback, because a page that yields
  // nothing simply leaves the feed as it was.
  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    var cursor = olderThan;
    final filtering = excludeWatched && await _isEnabled();
    for (var dig = 0; dig < _maxPageDigs; dig += 1) {
      final page = await _feed.loadOlderFeed(kind, olderThan: cursor);
      if (!filtering) return page;
      final watched = await _watchedIndex();
      final fresh =
          page.posts.where((post) => !watched.contains(post)).toList();
      if (fresh.isNotEmpty || !page.hasMore) {
        return VideoFeedPage(posts: fresh, nextOlderThan: page.nextOlderThan);
      }
      cursor = page.nextOlderThan!;
    }
    return VideoFeedPage(posts: const <VideoPost>[], nextOlderThan: cursor);
  }

  // Only reached when every fetched video is already watched: the feed must
  // not dead-end, so repeat the pool starting from the videos watched
  // longest ago.
  List<VideoPost> _leastRecentlyWatched(
    List<VideoPost> posts,
    WatchedVideoIndex watched,
  ) {
    final ordered = posts.toList()
      ..sort((left, right) {
        return watched.watchedAt(left)!.compareTo(watched.watchedAt(right)!);
      });
    return List<VideoPost>.unmodifiable(ordered);
  }

  Future<bool> _isEnabled() async {
    try {
      return (await _settings.load()).hideWatchedVideos;
    } on Object catch (error, stackTrace) {
      _report('WatchAwareVideoFeedRepository.settings', error, stackTrace);
      return false;
    }
  }

  Future<WatchedVideoIndex> _watchedIndex() async {
    try {
      return WatchedVideoIndex(
        await _history.snapshotForActiveAccount().load(),
      );
    } on Object catch (error, stackTrace) {
      _report('WatchAwareVideoFeedRepository.history', error, stackTrace);
      return WatchedVideoIndex(const <WatchHistoryEntry>[]);
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
