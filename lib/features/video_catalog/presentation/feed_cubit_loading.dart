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
    final roster = _session.loaded(fresh);
    _backfill.restartFrom(roster.posts);
    if (roster.isEmpty) return _emitEmpty(kind);
    _emitState(FeedLoaded.of(kind, roster));
    _hunt.filled();
    _viewer.landedOn(roster.posts, 0);
    _ensureBuffered();
  }

  void _acceptRefresh(FeedLoaded initial, List<VideoPost> refreshed) {
    final current = state is FeedLoaded ? state as FeedLoaded : initial;
    final roster = _session.resynced(current.roster, refreshed);
    if (roster.isEmpty) return _emitEmpty(current.kind);
    _emitState(FeedLoaded.of(current.kind, roster));
    _viewer.stayedOn(roster.active);
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
