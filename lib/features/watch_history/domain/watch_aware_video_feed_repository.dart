import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';

final class WatchAwareVideoFeedRepository
    implements VideoFeedRepository, VideoFeedRefreshRepository {
  const WatchAwareVideoFeedRepository({
    required VideoFeedRepository feed,
    required WatchHistoryRepository history,
    required FailureReporter failureReporter,
  }) : _feed = feed,
       _history = history,
       _failureReporter = failureReporter;

  static const _maxPageDigs = 3;

  final VideoFeedRepository _feed;
  final WatchHistoryRepository _history;
  final FailureReporter _failureReporter;

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    final history = _historyFor(kind);
    final all = await _feed.loadFeed(kind);
    final eligible = history == null ? all : await _fresh(all, history);
    return VideoFeedRefreshSnapshot(allPosts: all, eligiblePosts: eligible);
  }

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    final history = _historyFor(kind);
    final posts = await _feed.loadFeed(kind);
    if (history == null) return posts;
    final fresh = await _fresh(posts, history);
    if (_settlesPage(fresh, posts.isEmpty)) return fresh;
    return _digPastWatched(kind, posts, history);
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    final history = _historyFor(kind);
    var cursor = olderThan;
    for (var dig = 0; dig < _maxPageDigs; dig += 1) {
      final page = await _feed.loadOlderFeed(kind, olderThan: cursor);
      if (history == null) return page;
      final fresh = await _fresh(page.posts, history);
      if (_settlesPage(fresh, !page.hasMore)) {
        return VideoFeedPage(posts: fresh, nextOlderThan: page.nextOlderThan);
      }
      cursor = page.nextOlderThan!;
    }
    return VideoFeedPage(posts: const <VideoPost>[], nextOlderThan: cursor);
  }

  Future<List<VideoPost>> _digPastWatched(
    FeedKind kind,
    List<VideoPost> posts,
    WatchHistoryRepository history,
  ) async {
    var cursor = _oldestActivityAt(posts);
    for (var dig = 0; dig < _maxPageDigs; dig += 1) {
      final page = await _feed.loadOlderFeed(kind, olderThan: cursor);
      final fresh = await _fresh(page.posts, history);
      if (fresh.isNotEmpty) return fresh;
      if (!page.hasMore) break;
      cursor = page.nextOlderThan!;
    }
    return const <VideoPost>[];
  }

  Future<List<VideoPost>> _fresh(
    List<VideoPost> posts,
    WatchHistoryRepository history,
  ) async {
    try {
      return await history.filterUnwatched(posts);
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'WatchAwareVideoFeedRepository.history',
        error: error,
        stackTrace: stackTrace,
      );
      rethrow;
    }
  }

  WatchHistoryRepository? _historyFor(FeedKind kind) {
    if (kind == FeedKind.following) return null;
    try {
      return _history.snapshotForActiveAccount();
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'WatchAwareVideoFeedRepository.history',
        error: error,
        stackTrace: stackTrace,
      );
      rethrow;
    }
  }

  bool _settlesPage(List<VideoPost> fresh, bool exhausted) {
    return fresh.isNotEmpty || exhausted;
  }

  DateTime _oldestActivityAt(List<VideoPost> posts) {
    var oldest = posts.first.feedActivityAt;
    for (final post in posts.skip(1)) {
      if (post.feedActivityAt.isBefore(oldest)) oldest = post.feedActivityAt;
    }
    return oldest.subtract(const Duration(seconds: 1));
  }
}
