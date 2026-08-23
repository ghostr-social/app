part of 'feed_cubit.dart';

typedef _PageTransitionCommit = ({
  int transition,
  FeedLoaded current,
  int index,
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
    final commit = (
      transition: transition,
      current: current,
      index: index,
      decision: decision,
    );
    final preparation = _viewer.prepareToShow(current.posts[index]);
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
    final current = _acceptedPageTransition(commit.transition, commit.current);
    if (current == null) return;
    final moved = _movedTo(current, commit.index);
    emit(moved);
    _viewer.landedOn(moved.posts, moved.activeIndex);
    _rememberPendingRescue(current, moved, commit.decision);
    _rescueAfterDeliveryUpdate();
    _ensureBuffered();
  }

  FeedLoaded? _acceptedPageTransition(int transition, FeedLoaded from) {
    final current = state;
    if (isClosed || transition != _pageTransition || current is! FeedLoaded) {
      return null;
    }
    if (current.activeIndex != from.activeIndex) return null;
    return identical(current.rosterRevision, from.rosterRevision)
        ? current
        : null;
  }

  bool _acceptsExactPageTransition(int transition, FeedLoaded from) {
    final current = _acceptedPageTransition(transition, from);
    return current != null && identical(current.posts, from.posts);
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
