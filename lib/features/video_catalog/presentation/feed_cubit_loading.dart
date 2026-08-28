part of 'feed_cubit.dart';

typedef _RefreshLists = ({List<VideoPost> refreshed, List<VideoPost> eligible});

typedef _RosterProposal = ({FeedRoster roster, VideoInteractionTarget? target});
typedef _RefreshProposal = ({
  FeedRoster roster,
  VideoInteractionTarget? target,
  FeedSessionResync refresh,
});

extension FeedCubitLoading on FeedCubit {
  Future<void> _accepting(
    FeedKind kind,
    FeedState Function(String reason) unavailable,
  ) async {
    final answer = await _loads.leased(() => _fetch.unwatched(kind));
    if (isClosed || answer == null) return;
    switch (answer.value) {
      case FeedUnavailable(:final failure):
        _emitState(unavailable(feedLoadFailureMessage(failure)));
      case FeedFetched(:final posts):
        if (posts.isNotEmpty) _acknowledgePendingFeedUpdate();
        await _acceptLoad(kind, posts, answer.request);
    }
  }

  Future<void> _acceptLoad(
    FeedKind kind,
    List<VideoPost> fresh,
    int request,
  ) async {
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
    int request,
  ) async {
    final refreshTransition = _pageTransition;
    await _awaitPageTransition(refreshTransition);
    if (refreshTransition != _pageTransition || !_loads.accepts(request)) {
      return;
    }
    final current = state is FeedLoaded ? state as FeedLoaded : initial;
    final lists = _includeRefreshTail(initial, current, (
      refreshed: refreshed,
      eligible: eligible,
    ));
    await _runRefreshTransaction(current, lists, request);
  }

  Future<void> _runRefreshTransaction(
    FeedLoaded current,
    _RefreshLists initialLists,
    int request,
  ) async {
    var basis = current;
    var lists = initialLists;
    var proposal = _refreshProposal(current, lists);
    if (proposal.roster.isEmpty) return _commitRefresh(basis, proposal);
    final transition = ++_pageTransition;
    while (!proposal.roster.isEmpty) {
      if (!await _viewer.prepareToShow(proposal.roster.active)) return;
      final latest = _acceptedRefreshTransition(transition, basis, request);
      if (latest == null) return;
      lists = _includeRefreshTail(basis, latest, lists);
      final rebased = _refocusRefresh(latest, lists, proposal);
      if (rebased.target == proposal.target) {
        return _commitRefresh(latest, rebased);
      }
      basis = latest;
      proposal = rebased;
    }
    _commitRefresh(basis, proposal);
  }

  FeedLoaded? _acceptedRefreshTransition(
    int transition,
    FeedLoaded basis,
    int request,
  ) {
    if (!_loads.accepts(request)) return null;
    return _acceptedPageTransition(transition, basis);
  }

  _RefreshProposal _refreshProposal(FeedLoaded current, _RefreshLists lists) {
    final refresh = _session.captureResync(
      lists.refreshed,
      eligible: lists.eligible,
      retainWatched: !_excludesWatched(current),
    );
    final roster = _session.previewResynced(current.roster, refresh);
    return (
      roster: roster,
      target: roster.isEmpty
          ? null
          : VideoInteractionTarget.fromPost(roster.active),
      refresh: refresh,
    );
  }

  _RefreshProposal _refocusRefresh(
    FeedLoaded current,
    _RefreshLists lists,
    _RefreshProposal previous,
  ) {
    final proposal = _refreshProposal(current, lists);
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
      refresh: proposal.refresh,
    );
  }

  _RefreshLists _includeRefreshTail(
    FeedLoaded before,
    FeedLoaded after,
    _RefreshLists lists,
  ) {
    if (!identical(before.rosterRevision, after.rosterRevision) ||
        after.posts.length <= before.posts.length) {
      return lists;
    }
    final tail = after.posts.sublist(before.posts.length);
    return (
      refreshed: FeedPagination.appendNew(lists.refreshed, tail),
      eligible: FeedPagination.appendNew(lists.eligible, tail),
    );
  }

  void _commitRefresh(FeedLoaded current, _RefreshProposal proposal) {
    var roster = _session.resynced(current.roster, proposal.refresh);
    if (roster.isEmpty) return _emitEmpty(current.kind);
    final target = proposal.target;
    final index = target == null ? -1 : _targetIndex(roster, target);
    if (index < 0) return;
    roster = _session.positionedAt(
      roster,
      index,
      history: FeedNavigationHistory.unlimited,
    );
    final loaded = FeedLoaded.of(current.kind, roster, follows: _follows);
    _emitState(_projectPreparation(loaded));
    unawaited(_settleReposts());
    _viewer.rosterChanged(roster.posts, roster.activeIndex);
    _ensureBuffered();
  }

  void _emitEmpty(FeedKind kind) {
    _cancelPageTransition();
    _emitState(FeedEmpty(kind));
    _hunt.emptied(_startHuntAttempt);
  }

  Future<void> _huntEmptyFeed() async {
    final current = state;
    if (current is! FeedEmpty) return;
    await _runFeedPull(() async {
      await _ensureFeedUpdates(current.kind);
      final answer = await _loads.leased(() => _fetch.unwatched(current.kind));
      if (isClosed || answer == null) return;
      if (answer.value case FeedFetched(:final posts) when posts.isNotEmpty) {
        _acknowledgePendingFeedUpdate();
        return _acceptLoad(current.kind, posts, answer.request);
      }
      _hunt.emptied(_startHuntAttempt);
    });
  }

  void _startHuntAttempt() => unawaited(_huntEmptyFeed());
}
