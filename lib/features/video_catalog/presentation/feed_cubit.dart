import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_like_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_interaction_reconciler.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_pagination.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_inventory/domain/feed_media_prefetcher.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

export 'feed_state.dart';

class FeedDependencies {
  const FeedDependencies({
    required this.feed,
    required this.engagement,
    this.social,
    this.prefetcher,
    this.watchTracker,
  });

  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
  final SocialGraphRepository? social;
  final FeedMediaPrefetcher? prefetcher;
  final WatchHistoryTracker? watchTracker;
}

class FeedCubit extends DisposalSafeCubit<FeedState> {
  FeedCubit(this._dependencies) : super(const FeedLoading(FeedKind.forYou));

  final FeedDependencies _dependencies;
  final _interactions = FeedInteractionReconciler();
  final _pagination = FeedPagination();
  static const _likePolicy = VideoLikePolicy();
  static const _loadMoreThreshold = 5;
  static const _minLoadedPosts = 10;
  List<VideoPost> _lastPosts = const <VideoPost>[];
  int _loadRequest = 0;

  Future<void> load([FeedKind? selectedKind]) async {
    final request = ++_loadRequest;
    final kind = selectedKind ?? state.kind;
    emit(FeedLoading(kind));
    try {
      final posts =
          await _dependencies.feed.loadFeed(kind, excludeWatched: true);
      _acceptLoad(request, kind, posts);
    } on AppFailure catch (failure) {
      _emitLoad(request, FeedFailure(kind, failure.message));
    } on Object catch (error, stackTrace) {
      _emitLoad(
        request,
        FeedFailure(kind, _unexpectedLoad(error, stackTrace)),
      );
    }
  }

  Future<void> retry() => load();

  Future<void> refresh() {
    final current = state;
    if (current is! FeedLoaded) return load();
    return _refreshLoaded(current);
  }

  Future<void> reload() async {
    final previous = state;
    if (previous is! FeedLoaded) return load();
    final request = ++_loadRequest;
    emit(FeedLoading(previous.kind));
    try {
      final posts = await _dependencies.feed
          .loadFeed(previous.kind, excludeWatched: true);
      _acceptLoad(request, previous.kind, posts);
    } on AppFailure catch (failure) {
      _emitLoad(request, previous.withNotice(failure.message));
    } on Object catch (error, stackTrace) {
      _emitLoad(
          request, previous.withNotice(_unexpectedLoad(error, stackTrace)));
    }
  }

  Future<void> _refreshLoaded(FeedLoaded current) async {
    final request = ++_loadRequest;
    try {
      final posts = await _dependencies.feed.loadFeed(current.kind);
      _acceptRefresh(request, current, posts);
    } on AppFailure catch (failure) {
      _emitLoad(request, current.withNotice(failure.message));
    } on Object catch (error, stackTrace) {
      _emitLoad(
          request, current.withNotice(_unexpectedLoad(error, stackTrace)));
    }
  }

  void _acceptRefresh(
    int request,
    FeedLoaded initial,
    List<VideoPost> refreshed,
  ) {
    if (!_acceptsLoad(request)) return;
    final current = state is FeedLoaded ? state as FeedLoaded : initial;
    final reconciled = _interactions.reconcile(
      refreshed: refreshed,
      current: current.posts,
    );
    final posts = _sessionPosts(current.posts, reconciled);
    _lastPosts = posts;
    final next = _refreshedState(current, posts);
    emit(next);
    if (next is FeedLoaded) _trackWatched(next.posts[next.activeIndex]);
  }

  // A refresh resynchronizes the posts the viewer is already scrolling
  // through; it never inserts content above their position. Fresh content
  // arrives only via load()/reload().
  List<VideoPost> _sessionPosts(
    List<VideoPost> current,
    List<VideoPost> reconciled,
  ) {
    final byTarget = <VideoInteractionTarget, VideoPost>{
      for (final post in reconciled)
        VideoInteractionTarget.fromPost(post): post,
    };
    return [
      for (final post in current)
        if (byTarget[VideoInteractionTarget.fromPost(post)] case final post?)
          post,
    ];
  }

  void _acceptLoad(int request, FeedKind kind, List<VideoPost> refreshed) {
    if (!_acceptsLoad(request)) return;
    final posts = _interactions.reconcile(
      refreshed: refreshed,
      current: _lastPosts,
    );
    _lastPosts = posts;
    _pagination.restartFrom(posts);
    emit(posts.isEmpty ? FeedEmpty(kind) : FeedLoaded(kind, posts));
    if (posts.isNotEmpty) {
      _trackWatched(posts.first);
      _dependencies.prefetcher?.focus(posts, 0);
    }
    if (posts.isNotEmpty && posts.length < _minLoadedPosts) {
      unawaited(loadMore());
    }
  }

  FeedState _refreshedState(FeedLoaded current, List<VideoPost> posts) {
    if (posts.isEmpty) return FeedEmpty(current.kind);
    return FeedLoaded(
      current.kind,
      posts,
      activeIndex: _preservedIndex(current, posts),
    );
  }

  int _preservedIndex(FeedLoaded current, List<VideoPost> posts) {
    final active = VideoInteractionTarget.fromPost(
      current.posts[current.activeIndex],
    );
    final index = posts.indexWhere(
      (post) => VideoInteractionTarget.fromPost(post) == active,
    );
    if (index >= 0) return index;
    final lastIndex = posts.length - 1;
    return current.activeIndex > lastIndex ? lastIndex : current.activeIndex;
  }

  void pageChanged(int index) {
    final current = state;
    if (current is! FeedLoaded) return;
    if (index < 0 || index >= current.posts.length) return;
    emit(current.withPage(index));
    _trackWatched(current.posts[index]);
    _dependencies.prefetcher?.focus(current.posts, index);
    if (current.posts.length - index <= _loadMoreThreshold) {
      unawaited(loadMore());
    }
  }

  Future<void> loadMore() async {
    final current = state;
    if (current is! FeedLoaded) return;
    final cursor = _pagination.beginLoad();
    if (cursor == null) return;
    final request = _loadRequest;
    try {
      final page = await _dependencies.feed.loadOlderFeed(
        current.kind,
        olderThan: cursor,
        excludeWatched: true,
      );
      if (request != _loadRequest) return;
      _pagination.completeLoad(page);
      _appendPage(page);
    } on AppFailure catch (failure) {
      _pagination.failLoad();
      _showNotice(failure.message);
    } on Object catch (error, stackTrace) {
      _pagination.failLoad();
      _showNotice(_unexpectedLoad(error, stackTrace));
    }
  }

  void _appendPage(VideoFeedPage page) {
    final current = state;
    if (page.posts.isEmpty || current is! FeedLoaded) return;
    final reconciled = _interactions.reconcile(
      refreshed: page.posts,
      current: const <VideoPost>[],
    );
    final posts = FeedPagination.appendNew(current.posts, reconciled);
    if (posts.length == current.posts.length) return;
    _lastPosts = posts;
    emit(current.withPosts(posts));
  }

  void _trackWatched(VideoPost post) {
    final tracker = _dependencies.watchTracker;
    if (tracker != null) unawaited(tracker.videoWatched(post));
  }

  // The heart flips immediately; the relay mutation confirms or reverts it.
  Future<void> toggleLike(VideoPost post) async {
    _acceptUpdatedPost(_likePolicy.toggle(post));
    try {
      final updated = await _dependencies.engagement.toggleLike(post);
      _acceptUpdatedPost(updated);
    } on AppFailure catch (failure) {
      _revertLike(post, failure.message);
    } on Object catch (error, stackTrace) {
      _revertLike(post, _unexpectedLike(error, stackTrace));
    }
  }

  void _revertLike(VideoPost original, String message) {
    _acceptUpdatedPost(original);
    _showNotice(message);
  }

  Future<void> blockCreator(VideoPost post) async {
    final social = _dependencies.social;
    if (social == null) return;
    try {
      final isBlocked = await social.toggleBlock(post.creator.id);
      if (isBlocked) await _removeBlockedCreator(post);
    } on AppFailure catch (failure) {
      _showNotice(failure.message);
    } on Object catch (error, stackTrace) {
      _showNotice(_unexpectedBlock(error, stackTrace));
    }
  }

  Future<void> _removeBlockedCreator(VideoPost post) async {
    final current = state;
    _lastPosts = _withoutCreator(_lastPosts, post);
    if (current is! FeedLoaded) return;
    final remaining = _withoutCreator(current.posts, post);
    if (remaining.isEmpty) return load();
    emit(FeedLoaded(
      current.kind,
      remaining,
      activeIndex: _survivingIndex(current, remaining),
      notice: 'Blocked ${post.creator.handle}',
    ));
  }

  List<VideoPost> _withoutCreator(List<VideoPost> posts, VideoPost blocked) {
    return [
      for (final post in posts)
        if (post.creator.id != blocked.creator.id) post,
    ];
  }

  // The first surviving post at or below the viewer's position keeps playing.
  int _survivingIndex(FeedLoaded current, List<VideoPost> remaining) {
    for (var index = current.activeIndex;
        index < current.posts.length;
        index += 1) {
      final target = VideoInteractionTarget.fromPost(current.posts[index]);
      final found = remaining.indexWhere(
        (post) => VideoInteractionTarget.fromPost(post) == target,
      );
      if (found >= 0) return found;
    }
    return remaining.length - 1;
  }

  void commentsPublished(VideoPost post, int publishedCount) {
    if (publishedCount < 1) return;
    final current = state;
    final posts = _interactions.acceptComments(
      post,
      publishedCount,
      current is FeedLoaded ? current.posts : _lastPosts,
    );
    _lastPosts = posts;
    if (current is FeedLoaded) emit(current.withPosts(posts));
  }

  void clearNotice() {
    final current = state;
    if (current is FeedLoaded && current.notice != null) {
      emit(current.withoutNotice());
    }
  }

  void _acceptUpdatedPost(VideoPost updated) {
    final current = state;
    final posts = _interactions.acceptLike(
      updated,
      current is FeedLoaded ? current.posts : _lastPosts,
    );
    _lastPosts = posts;
    if (current is FeedLoaded) emit(current.withPosts(posts));
  }

  void _showNotice(String message) {
    final current = state;
    if (current is FeedLoaded) emit(current.withNotice(message));
  }

  void _emitLoad(int request, FeedState next) {
    if (_acceptsLoad(request)) emit(next);
  }

  bool _acceptsLoad(int request) => !isClosed && request == _loadRequest;

  String _unexpectedLoad(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'FeedCubit.load',
      message: 'Could not load the Nostr video feed.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  String _unexpectedLike(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'FeedCubit.toggleLike',
      message: 'Could not update this like.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  String _unexpectedBlock(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'FeedCubit.blockCreator',
      message: 'Could not block this creator.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
