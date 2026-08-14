import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/settings/domain/app_settings_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watched_video_index.dart';

class WatchAwareVideoFeedRepository
    implements VideoFeedRepository, VideoFeedRefreshRepository {
  const WatchAwareVideoFeedRepository({
    required VideoFeedRepository feed,
    required WatchHistoryRepository history,
    required AppSettingsRepository settings,
    required FailureReporter failureReporter,
  }) : _feed = feed,
       _history = history,
       _settings = settings,
       _failureReporter = failureReporter;

  final VideoFeedRepository _feed;
  final WatchHistoryRepository _history;
  final AppSettingsRepository _settings;
  final FailureReporter _failureReporter;

  static const _maxPageDigs = 3;

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    final all = await _feed.loadFeed(kind);
    if (!await _isEnabled()) {
      return VideoFeedRefreshSnapshot(allPosts: all, eligiblePosts: all);
    }
    final watched = await _watchedIndex();
    return VideoFeedRefreshSnapshot(
      allPosts: all,
      eligiblePosts: _fresh(all, watched),
    );
  }

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final posts = await _feed.loadFeed(kind);
    if (!excludeWatched || !await _isEnabled()) return posts;
    final watched = await _watchedIndex();
    if (watched.isEmpty) return posts;
    final fresh = _fresh(posts, watched);
    if (fresh.isNotEmpty || posts.isEmpty) return fresh;
    return _digPastWatched(kind, posts, watched);
  }

  // Fully-watched pages are skipped by digging further into the past; a
  // page that yields nothing simply leaves the feed as it was.
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
      final fresh = _fresh(page.posts, watched);
      if (fresh.isNotEmpty || !page.hasMore) {
        return VideoFeedPage(posts: fresh, nextOlderThan: page.nextOlderThan);
      }
      cursor = page.nextOlderThan!;
    }
    return VideoFeedPage(posts: const <VideoPost>[], nextOlderThan: cursor);
  }

  // A watched video is never served again, so a fully watched snapshot
  // digs into the past for unseen ones and an exhausted feed simply comes
  // back empty for the empty-feed hunt to refill.
  Future<List<VideoPost>> _digPastWatched(
    FeedKind kind,
    List<VideoPost> posts,
    WatchedVideoIndex watched,
  ) async {
    var cursor = _oldestPublishedAt(posts);
    for (var dig = 0; dig < _maxPageDigs; dig += 1) {
      final page = await _feed.loadOlderFeed(kind, olderThan: cursor);
      final fresh = _fresh(page.posts, watched);
      if (fresh.isNotEmpty) return fresh;
      if (!page.hasMore) break;
      cursor = page.nextOlderThan!;
    }
    return const <VideoPost>[];
  }

  List<VideoPost> _fresh(List<VideoPost> posts, WatchedVideoIndex watched) {
    return List<VideoPost>.unmodifiable(
      posts.where((post) => !watched.contains(post)),
    );
  }

  DateTime _oldestPublishedAt(List<VideoPost> posts) {
    var oldest = posts.first.publishedAt;
    for (final post in posts.skip(1)) {
      if (post.publishedAt.isBefore(oldest)) oldest = post.publishedAt;
    }
    return oldest.subtract(const Duration(seconds: 1));
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
