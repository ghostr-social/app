part of 'feed_cubit.dart';

typedef _PageTransitionCommit = ({
  int transition,
  FeedLoaded current,
  FeedLoaded moved,
  FeedReadyDecision decision,
});

extension FeedCubitNavigation on FeedCubit {
  void pageChanged(int index) {
    final current = state;
    if (current is! FeedLoaded) return;
    if (!_containsPage(current, index)) return;
    if (_consumeTransportJump(index)) return;
    final transition = ++_pageTransition;
    final decision = _readyDecision(current, index);
    if (decision.action == FeedReadyAction.rescue) {
      return _rescueTo(current, decision, transition);
    }
    _preparePageTransition(current, index, transition, decision);
  }

  void _preparePageTransition(
    FeedLoaded current,
    int index,
    int transition,
    FeedReadyDecision decision,
  ) {
    _clearPendingRescue();
    final moved = _movedTo(current, index);
    final commit = (
      transition: transition,
      current: current,
      moved: moved,
      decision: decision,
    );
    final preparation = _viewer.prepareToShow(moved.roster.active);
    if (preparation is Future<bool>) {
      return unawaited(_finishPageTransition(preparation, commit));
    }
    if (!preparation) return;
    _finishPageTransitionNow(commit);
  }

  bool _containsPage(FeedLoaded current, int index) {
    return index >= 0 && index < current.posts.length;
  }

  Future<void> _finishPageTransition(
    Future<bool> preparation,
    _PageTransitionCommit commit,
  ) async {
    if (!await preparation) return;
    _finishPageTransitionNow(commit);
  }

  void _finishPageTransitionNow(_PageTransitionCommit commit) {
    if (!_acceptsPageTransition(commit.transition, commit.current)) return;
    emit(commit.moved);
    _viewer.landedOn(commit.moved.posts, commit.moved.activeIndex);
    _rememberPendingRescue(commit.current, commit.moved, commit.decision);
    _rescueAfterDeliveryUpdate();
    _ensureBuffered();
  }

  bool _acceptsPageTransition(int transition, FeedLoaded from) {
    return !isClosed && transition == _pageTransition && identical(state, from);
  }

  FeedLoaded _movedTo(FeedLoaded current, int index) {
    final roster = _session.movedTo(
      current.roster,
      index,
      forgetPrevious: _forgetsViewed(current),
    );
    final moved = FeedLoaded.of(
      current.kind,
      roster,
      notice: current.notice,
      follows: current.follows,
    );
    return _realignPreparation(current, moved);
  }

  bool _forgetsViewed(FeedLoaded current) {
    return _dependencies.watchTracker != null &&
        _dependencies.replayPolicy == FeedReplayPolicy.prevent &&
        current.kind != FeedKind.following;
  }

  void _surfaceVisibilityChanged(bool isVisible) {
    _viewer.visibilityChanged(isVisible);
    if (!isVisible) return _forgetExitedSurface();
    if (_reloadWhenSurfaceVisible) unawaited(load());
  }

  void _forgetExitedSurface() {
    if (_dependencies.watchTracker == null) return;
    _reloadWhenSurfaceVisible = true;
    _loads.take();
    _pageTransition += 1;
    _clearPendingRescue();
    _hunt.filled();
    if (state is! FeedLoading) emit(FeedLoading(state.kind));
  }

  void clearNotice() {
    final current = state;
    if (current is! FeedLoaded || current.notice == null) return;
    emit(current.withoutNotice());
  }
}
