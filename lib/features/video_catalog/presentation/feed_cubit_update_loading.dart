part of 'feed_cubit.dart';

extension FeedCubitUpdateLoading on FeedCubit {
  Future<void> refresh() async {
    if (!_isSurfaceVisible) {
      _refreshWhenSurfaceVisible = true;
      return;
    }
    _refreshWhenSurfaceVisible = false;
    final previous = state;
    if (previous is! FeedLoaded) return load();
    final transition = _pageTransition;
    _reposts?.forget();
    final follows = _reloadFollows();
    await _runFeedPull(() => _refreshLoaded(previous, transition));
    await follows;
  }

  Future<void> _refreshLoaded(FeedLoaded previous, int transition) async {
    await _refreshFeedUpdates(previous.kind);
    final through = _updates.revision;
    final result = await _loads.newest(() => _fetch.resync(previous.kind));
    final accepted = _currentRefresh(transition, result);
    if (accepted == null) return;
    _applyManualRefresh(previous, accepted, through);
  }

  FeedFetch? _currentRefresh(int transition, FeedFetch? result) {
    if (isClosed || transition != _pageTransition || result == null) {
      return null;
    }
    return result;
  }

  void _applyManualRefresh(
    FeedLoaded previous,
    FeedFetch result,
    BigInt through,
  ) {
    switch (result) {
      case FeedUnavailable():
        final current = state;
        if (current is FeedLoaded) {
          emit(
            current
                .withFollows(_follows)
                .withNotice(feedLoadFailureMessage(result.failure)),
          );
        }
      case FeedFetched(
        :final posts,
        :final eligiblePosts,
        :final blockedProfiles,
      ):
        final applied = _acceptRefresh(
          previous,
          posts,
          eligiblePosts,
          blockedProfiles,
        );
        if (applied) _acknowledgePendingFeedUpdate(through);
    }
  }

  Future<bool> _reloadFromFeedUpdate(
    int feed,
    FeedKind kind,
    bool allowEmpty,
  ) async {
    final transition = _pageTransition;
    final result = await _loadFeedUpdate(kind);
    if (transition != _pageTransition || result == null) return false;
    final accepted = _acceptedFeedUpdate(feed, kind, allowEmpty, result);
    if (accepted == null) return true;
    return _applyFeedUpdate(kind, accepted);
  }

  Future<FeedFetch?> _loadFeedUpdate(FeedKind kind) {
    final loaded = state is FeedLoaded;
    return _loads.newest(
      () => loaded ? _fetch.resync(kind) : _fetch.unwatched(kind),
    );
  }

  FeedFetched? _acceptedFeedUpdate(
    int feed,
    FeedKind kind,
    bool allowEmpty,
    FeedFetch? result,
  ) {
    if (!_acceptsFeedReconciliation(feed, kind)) return null;
    if (result is! FeedFetched) return null;
    return result.posts.isNotEmpty || allowEmpty ? result : null;
  }

  Future<bool> _applyFeedUpdate(FeedKind kind, FeedFetched accepted) async {
    final FeedFetched(:posts, :eligiblePosts, :blockedProfiles) = accepted;
    final current = state;
    if (current is FeedLoaded) {
      return _acceptRefresh(current, posts, eligiblePosts, blockedProfiles);
    }
    await _acceptLoad(kind, posts);
    return state is FeedLoaded || state is FeedEmpty;
  }
}
