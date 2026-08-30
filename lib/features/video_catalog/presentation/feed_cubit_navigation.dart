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
    _beginPageTransition(transition);
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
    if (!preparation) return _completePageTransition(transition);
    _finishPageTransitionNow(commit);
  }

  bool _containsPage(FeedLoaded current, int index) {
    return index >= 0 && index < current.posts.length;
  }

  Future<void> _finishPageTransition(
    Future<bool> preparation,
    _PageTransitionCommit commit,
  ) async {
    if (!await preparation) return _completePageTransition(commit.transition);
    _finishPageTransitionNow(commit);
  }

  void _finishPageTransitionNow(_PageTransitionCommit commit) {
    final current = _acceptedPageTransition(commit.transition, commit.current);
    if (current == null) return _completePageTransition(commit.transition);
    final moved = _presentedAt(current, commit.index);
    emit(moved);
    _completePageTransition(commit.transition);
    _viewer.landedOn(moved.posts, moved.activeIndex);
    _rememberPendingRescue(current, commit.decision);
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

  FeedLoaded _presentedAt(FeedLoaded current, int index) {
    final roster = _session.presentedAt(
      current.roster,
      index,
      history: _navigationHistory(current),
    );
    return _withRoster(current, roster);
  }

  FeedLoaded _withRoster(FeedLoaded current, FeedRoster roster) {
    final moved = FeedLoaded.of(
      current.kind,
      roster,
      notice: current.notice,
      follows: current.follows,
    );
    return _realignHls(current, _realignPreparation(current, moved));
  }

  bool _excludesWatched(FeedLoaded current) {
    return _dependencies.watchTracker != null &&
        _dependencies.replayPolicy == FeedReplayPolicy.prevent &&
        current.kind != FeedKind.following;
  }

  FeedNavigationHistory _navigationHistory(FeedLoaded current) {
    return _excludesWatched(current)
        ? FeedNavigationHistory.ordinary
        : FeedNavigationHistory.unlimited;
  }

  int _targetIndex(FeedRoster roster, VideoInteractionTarget target) {
    return roster.posts.indexWhere(
      (post) => VideoInteractionTarget.fromPost(post) == target,
    );
  }

  void _beginPageTransition(int transition) {
    _cancelPageTransition();
    _pendingPageTransition = _PageTransitionBarrier(transition);
  }

  void _completePageTransition(int transition) {
    final pending = _pendingPageTransition;
    if (pending?.transition != transition) return;
    _pendingPageTransition = null;
    pending?.complete();
  }

  void _cancelPageTransition() {
    final pending = _pendingPageTransition;
    _pendingPageTransition = null;
    pending?.complete();
  }

  Future<void> _awaitPageTransition(int transition) async {
    final pending = _pendingPageTransition;
    if (pending?.transition == transition) await pending!.completed;
  }

  Future<void> _awaitNavigationSettlement() async {
    while (true) {
      final pending = _pendingPageTransition;
      if (pending == null) return;
      await pending.completed;
    }
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
    _cancelPageTransition();
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

final class _PageTransitionBarrier {
  _PageTransitionBarrier(this.transition);

  final int transition;
  final Completer<void> _completion = Completer<void>();

  Future<void> get completed => _completion.future;

  void complete() {
    if (!_completion.isCompleted) _completion.complete();
  }
}
