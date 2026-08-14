part of 'feed_cubit.dart';

/// Engagement intents orchestrated by [FeedCubit].
extension FeedCubitEngagementActions on FeedCubit {
  Future<void> toggleLike(VideoPost post) async {
    _applyPosts(_session.liked(_visiblePosts, _engagement.optimistic(post)));
    final settled = await _engagement.confirmLike(post);
    _applyPosts(_session.liked(_visiblePosts, settled.post));
    if (settled.failure case final failure?) {
      _showNotice(feedLikeFailureMessage(failure));
    }
  }

  Future<void> blockCreator(VideoPost post) async {
    final result = await _engagement.block(post);
    if (result is FeedBlockFailed) {
      return _showNotice(feedBlockFailureMessage(result.failure));
    }
    if (result is FeedCreatorBlocked) await _removeBlockedCreator(post);
  }

  Future<void> _removeBlockedCreator(VideoPost post) async {
    final current = state;
    _session.dropCreator(post.creator.id);
    if (current is! FeedLoaded) return;
    final roster = current.roster.withoutCreator(post.creator.id);
    if (roster.isEmpty) return load();
    final blocked = 'Blocked ${post.creator.handle}';
    _emitState(
      FeedLoaded.of(current.kind, roster, notice: blocked, follows: _follows),
    );
    _viewer.rosterChanged(roster.posts, roster.activeIndex);
  }

  void commentsPublished(VideoPost post, int publishedCount) {
    if (publishedCount < 1) return;
    _applyPosts(_session.commented(_visiblePosts, post, publishedCount));
  }

  void _applyPosts(List<VideoPost> posts) {
    final current = state;
    if (current is FeedLoaded) _emitState(current.withPosts(posts));
  }

  List<VideoPost> get _visiblePosts {
    final current = state;
    return current is FeedLoaded ? current.posts : _session.held;
  }
}
