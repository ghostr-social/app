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

  Future<void> toggleRepost(VideoPost post) async {
    if (!canRepost(post)) return;
    final reposts = _reposts!;
    _applyPosts(
      _session.projectedRepost(_visiblePosts, reposts.optimistic(post)),
    );
    final settled = await reposts.confirm(post);
    _applyPosts(_repostResult(settled));
    _showRepostFailure(settled.failure);
  }

  List<VideoPost> _repostResult(FeedRepost settled) {
    return settled.failure == null
        ? _session.acceptedRepost(_visiblePosts, settled.post)
        : _session.projectedRepost(_visiblePosts, settled.post);
  }

  void _showRepostFailure(FeedOperationFailure? failure) {
    if (failure != null) _showNotice(feedRepostFailureMessage(failure));
  }

  bool canRepost(VideoPost post) {
    return _dependencies.viewerId != null &&
        (_reposts?.supports(post) ?? false);
  }

  Future<void> _settleReposts() async {
    final reposts = _reposts;
    final current = state;
    if (_dependencies.viewerId == null ||
        reposts == null ||
        current is! FeedLoaded) {
      return;
    }
    final settled = await reposts.settle(current.posts);
    if (!_canApplySettledReposts(current.kind, settled)) return;
    _applyPosts(_session.settledReposts(_visiblePosts, settled));
  }

  bool _canApplySettledReposts(FeedKind kind, List<VideoPost> settled) {
    return !isClosed &&
        settled.isNotEmpty &&
        state is FeedLoaded &&
        state.kind == kind;
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
    final transition = ++_pageTransition;
    if (!await _viewer.prepareToShow(roster.active)) return;
    if (!_acceptsPageTransition(transition, current)) return;
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
