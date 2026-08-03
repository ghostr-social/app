import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_interaction_reconciler.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

export 'feed_state.dart';

class FeedDependencies {
  const FeedDependencies({
    required this.feed,
    required this.engagement,
    this.watchTracker,
  });

  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
  final WatchHistoryTracker? watchTracker;
}

class FeedCubit extends DisposalSafeCubit<FeedState> {
  FeedCubit(this._dependencies) : super(const FeedLoading(FeedKind.forYou));

  final FeedDependencies _dependencies;
  final _interactions = FeedInteractionReconciler();
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
    emit(posts.isEmpty ? FeedEmpty(kind) : FeedLoaded(kind, posts));
    if (posts.isNotEmpty) _trackWatched(posts.first);
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
  }

  void _trackWatched(VideoPost post) {
    final tracker = _dependencies.watchTracker;
    if (tracker != null) unawaited(tracker.videoWatched(post));
  }

  Future<void> toggleLike(VideoPost post) async {
    try {
      final updated = await _dependencies.engagement.toggleLike(post);
      _acceptUpdatedPost(updated);
    } on AppFailure catch (failure) {
      _showNotice(failure.message);
    } on Object catch (error, stackTrace) {
      _showNotice(_unexpectedLike(error, stackTrace));
    }
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
}
