part of 'feed_cubit.dart';

typedef _NavigationCommit = ({
  int transition,
  FeedLoaded current,
  int index,
  VideoInteractionTarget target,
  FeedReadyDecision decision,
});

extension FeedCubitNavigation on FeedCubit {
  bool pageChanged(int index) {
    final current = state;
    if (current is! FeedLoaded) return false;
    if (!_containsPage(current, index)) return false;
    if (_consumeTransportJump(index)) return true;
    if (!_isSurfaceVisible) return index == current.activeIndex;
    final transition = ++_pageTransition;
    final decision = _readyDecision(current, index);
    if (decision.action == FeedReadyAction.rescue) {
      return _rescueTo(current, decision, transition);
    }
    return _preparePageTransition(current, index, transition, decision);
  }

  bool _preparePageTransition(
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
      index: index,
      target: VideoInteractionTarget.fromPost(moved.roster.active),
      decision: decision,
    );
    return _finishPageTransitionNow(commit);
  }

  bool _containsPage(FeedLoaded current, int index) {
    return index >= 0 && index < current.posts.length;
  }

  bool _finishPageTransitionNow(_NavigationCommit commit) {
    final current = _acceptedNavigation(commit);
    if (current == null) return false;
    final moved = _movedTo(current, commit.index);
    emit(moved);
    _viewer.landedOn(moved.posts, moved.activeIndex);
    _rememberPendingRescue(current, moved, commit.decision);
    _rescueAfterDeliveryUpdate();
    _ensureBuffered();
    return true;
  }

  FeedLoaded? _acceptedNavigation(_NavigationCommit commit) {
    if (isClosed || commit.transition != _pageTransition) return null;
    final current = state;
    if (current is! FeedLoaded || current.kind != commit.current.kind) {
      return null;
    }
    if (!_containsPage(current, commit.index)) return null;
    final target = VideoInteractionTarget.fromPost(current.posts[commit.index]);
    return target == commit.target ? current : null;
  }

  FeedLoaded _movedTo(FeedLoaded current, int index) {
    final roster = _session.movedTo(current.roster, index);
    return FeedLoaded.of(
      current.kind,
      roster,
      notice: current.notice,
      follows: current.follows,
    );
  }

  void _surfaceVisibilityChanged(bool isVisible) {
    _isSurfaceVisible = isVisible;
    _viewer.visibilityChanged(isVisible);
    if (!isVisible) return _suspendHiddenWork();
    _resumeVisibleViewer();
    _rescueAfterDeliveryUpdate();
    if (_reloadWhenSurfaceVisible) {
      _refreshWhenSurfaceVisible = false;
      unawaited(load());
      return;
    }
    if (_refreshWhenSurfaceVisible) {
      _refreshWhenSurfaceVisible = false;
      unawaited(refresh());
      return;
    }
    _startPendingFeedUpdate();
    if (state is FeedEmpty) _hunt.emptied(_startHuntAttempt);
    if (state is FeedLoaded) _ensureBuffered();
  }

  void _resumeVisibleViewer() {
    final current = state;
    if (current is! FeedLoaded) return;
    _viewer.rosterChanged(current.posts, current.activeIndex);
  }

  void _suspendHiddenWork() {
    _pageTransition += 1;
    _pausePendingRescue();
    _backfillRetry.cancel();
    _hunt.filled();
    if (state is FeedLoaded && _updates.pulls > 0) {
      _refreshWhenSurfaceVisible = true;
    }
    if (state is FeedLoading && _isPreparingLoad) return;
    _loads.take();
    if (state is FeedLoading || state is FeedFailure) {
      _reloadWhenSurfaceVisible = true;
    }
  }

  void _pausePendingRescue() {
    _rescueTimer?.cancel();
    _rescueTimer = null;
  }

  void clearNotice() {
    final current = state;
    if (current is! FeedLoaded || current.notice == null) return;
    emit(current.withoutNotice());
  }
}
