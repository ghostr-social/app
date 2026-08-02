import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
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
  int _loadRequest = 0;

  Future<void> load([FeedKind? selectedKind]) async {
    final request = ++_loadRequest;
    final kind = selectedKind ?? state.kind;
    emit(FeedLoading(kind));
    try {
      final posts = await _dependencies.feed.loadFeed(kind);
      _emitLoad(
          request, posts.isEmpty ? FeedEmpty(kind) : FeedLoaded(kind, posts));
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

  void clearNotice() {
    final current = state;
    if (current is FeedLoaded && current.notice != null) {
      emit(current.withoutNotice());
    }
  }

  void _acceptUpdatedPost(VideoPost updated) {
    final current = state;
    if (current is! FeedLoaded) return;
    final posts = current.posts
        .map((post) => post.id == updated.id ? updated : post)
        .toList();
    emit(current.withPosts(posts));
  }

  void _showNotice(String message) {
    final current = state;
    if (current is FeedLoaded) emit(current.withNotice(message));
  }

  void _emitLoad(int request, FeedState next) {
    if (!isClosed && request == _loadRequest) emit(next);
  }

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
