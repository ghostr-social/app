part of 'feed_cubit.dart';

extension FeedCubitUpdateLoading on FeedCubit {
  Future<void> refresh() async {
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
    final answer = await _loads.leased(() => _fetch.resync(previous.kind));
    final accepted = _currentRefresh(transition, answer);
    if (accepted == null) return;
    await _applyManualRefresh(previous, accepted);
  }

  FeedLoad<FeedFetch>? _currentRefresh(
    int transition,
    FeedLoad<FeedFetch>? answer,
  ) {
    if (isClosed || transition != _pageTransition || answer == null) {
      return null;
    }
    return answer;
  }

  Future<void> _applyManualRefresh(
    FeedLoaded previous,
    FeedLoad<FeedFetch> answer,
  ) async {
    switch (answer.value) {
      case FeedUnavailable(:final failure):
        emit(
          previous
              .withFollows(_follows)
              .withNotice(feedLoadFailureMessage(failure)),
        );
      case FeedFetched(:final posts, :final eligiblePosts):
        await _acceptRefresh(previous, posts, eligiblePosts, answer.request);
    }
  }

  Future<void> _reloadFromFeedUpdate(
    int feed,
    FeedKind kind,
    bool allowEmpty,
  ) async {
    final transition = _pageTransition;
    final result = await _loadFeedUpdate(kind);
    if (transition != _pageTransition) return;
    final accepted = _acceptedFeedUpdate(feed, kind, allowEmpty, result);
    if (accepted == null) return;
    await _applyFeedUpdate(kind, accepted);
  }

  Future<FeedLoad<FeedFetch>?> _loadFeedUpdate(FeedKind kind) {
    final loaded = state is FeedLoaded;
    return _loads.leased(
      () => loaded ? _fetch.resync(kind) : _fetch.unwatched(kind),
    );
  }

  FeedLoad<FeedFetch>? _acceptedFeedUpdate(
    int feed,
    FeedKind kind,
    bool allowEmpty,
    FeedLoad<FeedFetch>? answer,
  ) {
    if (!_acceptsFeedReconciliation(feed, kind)) return null;
    final result = answer?.value;
    if (answer == null || result is! FeedFetched) return null;
    return result.posts.isNotEmpty || allowEmpty ? answer : null;
  }

  Future<void> _applyFeedUpdate(
    FeedKind kind,
    FeedLoad<FeedFetch> accepted,
  ) async {
    final FeedFetched(:posts, :eligiblePosts) = accepted.value as FeedFetched;
    final current = state;
    if (current is FeedLoaded) {
      return _acceptRefresh(current, posts, eligiblePosts, accepted.request);
    }
    await _acceptLoad(kind, posts, accepted.request);
  }
}
