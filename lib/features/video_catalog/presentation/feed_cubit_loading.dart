part of 'feed_cubit.dart';

extension FeedCubitLoading on FeedCubit {
  Future<void> _accepting(
    FeedKind kind,
    FeedState Function(String reason) unavailable,
  ) async {
    final result = await _loads.newest(() => _fetch.unwatched(kind));
    if (isClosed || result == null) return;
    switch (result) {
      case FeedUnavailable():
        _emitState(unavailable(feedLoadFailureMessage(result.failure)));
      case FeedFetched(:final posts):
        if (posts.isNotEmpty) _acknowledgePendingFeedUpdate();
        await _acceptLoad(kind, posts);
    }
  }

  Future<void> _acceptLoad(FeedKind kind, List<VideoPost> fresh) async {
    final request = _loads.pending;
    final roster = _session.loaded(fresh).openedAt(_openAt);
    _backfillRetry.reset();
    _backfill.restartFrom(roster.posts);
    if (roster.isEmpty) {
      _emitEmpty(kind);
      return;
    }
    if (!await _viewer.prepareToShow(roster.active)) return;
    if (!_acceptsLoadedFeed(kind, request)) return;
    _emitState(
      _projectPreparation(FeedLoaded.of(kind, roster, follows: _follows)),
    );
    unawaited(_settleReposts());
    _hunt.filled();
    _viewer.landedOn(roster.posts, roster.activeIndex);
    _ensureBuffered();
  }

  bool _acceptsLoadedFeed(FeedKind kind, int request) {
    return !isClosed && state.kind == kind && _loads.accepts(request);
  }

  Future<void> _acceptRefresh(
    FeedLoaded initial,
    List<VideoPost> refreshed,
    List<VideoPost> eligible,
  ) async {
    final current = state is FeedLoaded ? state as FeedLoaded : initial;
    final roster = _session.resynced(
      current.roster,
      refreshed,
      eligible: eligible,
      retainWatched: !_forgetsViewed(current),
    );
    if (roster.isEmpty) {
      _emitEmpty(current.kind);
      return;
    }
    final transition = ++_pageTransition;
    if (!await _viewer.prepareToShow(roster.active)) return;
    if (!_acceptsPageTransition(transition, current)) return;
    final loaded = FeedLoaded.of(current.kind, roster, follows: _follows);
    _emitState(_projectPreparation(loaded));
    unawaited(_settleReposts());
    _viewer.rosterChanged(roster.posts, roster.activeIndex);
    _ensureBuffered();
  }

  void _emitEmpty(FeedKind kind) {
    _emitState(FeedEmpty(kind));
    _hunt.emptied(_startHuntAttempt);
  }

  Future<void> _huntEmptyFeed() async {
    final current = state;
    if (current is! FeedEmpty) return;
    await _runFeedPull(() async {
      await _ensureFeedUpdates(current.kind);
      final result = await _loads.newest(() => _fetch.unwatched(current.kind));
      if (isClosed || result == null) return;
      if (result case FeedFetched(:final posts) when posts.isNotEmpty) {
        _acknowledgePendingFeedUpdate();
        return _acceptLoad(current.kind, posts);
      }
      _hunt.emptied(_startHuntAttempt);
    });
  }

  void _startHuntAttempt() => unawaited(_huntEmptyFeed());
}
