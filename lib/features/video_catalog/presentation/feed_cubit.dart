import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_interaction_reconciler.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';

export 'feed_state.dart';

class FeedDependencies {
  const FeedDependencies({required this.feed, required this.engagement});

  final VideoFeedRepository feed;
  final VideoEngagementRepository engagement;
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
      final posts = await _dependencies.feed.loadFeed(kind);
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
    final posts = _interactions.reconcile(
      refreshed: refreshed,
      current: current.posts,
    );
    _lastPosts = posts;
    emit(_refreshedState(current, posts));
  }

  void _acceptLoad(int request, FeedKind kind, List<VideoPost> refreshed) {
    if (!_acceptsLoad(request)) return;
    final posts = _interactions.reconcile(
      refreshed: refreshed,
      current: _lastPosts,
    );
    _lastPosts = posts;
    emit(posts.isEmpty ? FeedEmpty(kind) : FeedLoaded(kind, posts));
  }

  FeedState _refreshedState(FeedLoaded current, List<VideoPost> posts) {
    if (posts.isEmpty) return FeedEmpty(current.kind);
    final lastIndex = posts.length - 1;
    final activeIndex = _preservedIndex(current.activeIndex, lastIndex);
    return FeedLoaded(current.kind, posts, activeIndex: activeIndex);
  }

  int _preservedIndex(int current, int lastIndex) {
    if (current > lastIndex) return lastIndex;
    return current;
  }

  void pageChanged(int index) {
    final current = state;
    if (current is FeedLoaded) emit(current.withPage(index));
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
