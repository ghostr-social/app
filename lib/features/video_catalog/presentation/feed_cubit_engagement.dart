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
    await _awaitNavigationSettlement();
    await _continueBlockedRemoval(post);
  }

  Future<void> _continueBlockedRemoval(VideoPost post) async {
    if (_isClosing || isClosed) return;
    if (_pendingPageTransition != null) return _removeBlockedCreator(post);
    final current = state;
    if (current is FeedLoaded) return _removeBlockedFrom(current, post);
    _session.dropCreator(post.creator.id);
  }

  Future<void> _removeBlockedFrom(FeedLoaded current, VideoPost post) async {
    final proposal = _blockedProposal(current, post.creator.id);
    if (proposal.roster.isEmpty) return _commitEmptyBlock(post.creator.id);
    final transition = ++_pageTransition;
    if (!await _viewer.prepareToShow(proposal.roster.active)) {
      return _retryBlockedRemoval(post);
    }
    final latest = _acceptedPageTransition(transition, current);
    if (latest == null) return _retryBlockedRemoval(post);
    final rebased = _refocusBlocked(latest, post.creator.id, proposal);
    if (rebased.target != proposal.target) {
      return _removeBlockedFrom(latest, post);
    }
    _commitBlocked(latest, post, rebased);
  }

  Future<void> _retryBlockedRemoval(VideoPost post) async {
    await _awaitNavigationSettlement();
    await _continueBlockedRemoval(post);
  }

  _RosterProposal _blockedProposal(FeedLoaded current, ProfileId creator) {
    final roster = current.roster.withoutCreator(creator);
    return (
      roster: roster,
      target: roster.isEmpty
          ? null
          : VideoInteractionTarget.fromPost(roster.active),
    );
  }

  _RosterProposal _refocusBlocked(
    FeedLoaded current,
    ProfileId creator,
    _RosterProposal previous,
  ) {
    final proposal = _blockedProposal(current, creator);
    final target = previous.target;
    if (target == null) return proposal;
    final index = _targetIndex(proposal.roster, target);
    if (index < 0) return proposal;
    return (
      roster: proposal.roster.movedTo(
        index,
        history: FeedNavigationHistory.unlimited,
      ),
      target: target,
    );
  }

  Future<void> _commitEmptyBlock(ProfileId creator) async {
    _session.dropCreator(creator);
    await load();
  }

  void _commitBlocked(
    FeedLoaded current,
    VideoPost post,
    _RosterProposal proposal,
  ) {
    _session.dropCreator(post.creator.id);
    var roster = current.roster.withoutCreator(post.creator.id);
    final target = proposal.target;
    final index = target == null ? -1 : _targetIndex(roster, target);
    if (index < 0) return;
    roster = _session.positionedAt(
      roster,
      index,
      history: FeedNavigationHistory.unlimited,
    );
    final blocked = 'Blocked ${post.creator.handle}';
    _emitState(
      _projectPreparation(
        FeedLoaded.of(current.kind, roster, notice: blocked, follows: _follows),
      ),
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
