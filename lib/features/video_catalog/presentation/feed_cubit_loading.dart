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
        _acceptLoad(kind, posts);
    }
  }

  void _acceptLoad(FeedKind kind, List<VideoPost> fresh) {
    final roster = _session.loaded(fresh).openedAt(_openAt);
    _backfillRetry.reset();
    _backfill.restartFrom(roster.posts);
    if (roster.isEmpty) return _emitEmpty(kind);
    _emitState(FeedLoaded.of(kind, roster, follows: _follows));
    _hunt.filled();
    _viewer.landedOn(roster.posts, roster.activeIndex);
    _ensureBuffered();
  }

  void _acceptRefresh(
    FeedLoaded initial,
    List<VideoPost> refreshed,
    List<VideoPost> eligible,
  ) {
    final current = state is FeedLoaded ? state as FeedLoaded : initial;
    final roster = _session.resynced(
      current.roster,
      refreshed,
      eligible: eligible,
    );
    if (roster.isEmpty) return _emitEmpty(current.kind);
    _emitState(FeedLoaded.of(current.kind, roster, follows: _follows));
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
