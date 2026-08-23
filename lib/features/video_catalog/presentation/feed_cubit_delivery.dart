part of 'feed_cubit.dart';

extension FeedCubitDelivery on FeedCubit {
  void _startDeliveryUpdates() {
    final updates = _dependencies.deliveryUpdates;
    if (updates == null) return;
    try {
      _deliverySubscription = updates.watchDelivery().listen(
        _acceptDeliveryUpdate,
        onError: _reportUpdateError,
        onDone: () => _deliverySubscription = null,
      );
    } on Object catch (error, stackTrace) {
      _reportUpdateError(error, stackTrace);
    }
  }

  void _acceptDeliveryUpdate(VideoDeliverySnapshot snapshot) {
    _delivery[snapshot.deliveryId] = snapshot;
    if (snapshot.phase == VideoDeliveryPhase.failed) {
      _rememberActiveFailure(snapshot.deliveryId);
    }
    _rescueAfterDeliveryUpdate();
  }

  bool _consumeTransportJump(int index) {
    final pending = _pendingTransportJump;
    _pendingTransportJump = null;
    return pending == index;
  }

  FeedReadyDecision _readyDecision(FeedLoaded current, int intended) {
    return _readySelector.select(
      current.posts,
      fromIndex: current.activeIndex,
      intendedIndex: intended,
      delivery: _delivery,
    );
  }

  void _rescueTo(
    FeedLoaded current,
    FeedReadyDecision decision,
    int transition,
  ) {
    _clearPendingRescue();
    final selected = decision.selectedIndex;
    final commit = _RescueCommit(transition, current, decision);
    final preparation = _viewer.prepareToShow(current.posts[selected]);
    if (preparation is Future<bool>) {
      return unawaited(_finishRescue(preparation, commit));
    }
    if (!preparation) return _completePageTransition(transition);
    _finishRescueNow(commit);
  }

  Future<void> _finishRescue(Future<bool> ready, _RescueCommit commit) async {
    if (!await ready) return _completePageTransition(commit.transition);
    _finishRescueNow(commit);
  }

  void _finishRescueNow(_RescueCommit commit) {
    final current = _acceptedPageTransition(commit.transition, commit.current);
    if (current == null) return _completePageTransition(commit.transition);
    final moved = _movedTo(current, commit.decision.selectedIndex);
    _pendingTransportJump = moved.activeIndex;
    emit(moved);
    _completePageTransition(commit.transition);
    _viewer.rescuedTo(
      moved.posts,
      moved.activeIndex,
      _transportRescue(commit.decision),
    );
    _ensureBuffered();
  }

  Future<void> _stopDeliveryUpdates() async {
    _clearPendingRescue();
    final subscription = _deliverySubscription;
    _deliverySubscription = null;
    await subscription?.cancel();
  }

  void _rescueAfterDeliveryUpdate() {
    final pending = _awaitingTransportRescue;
    if (pending == null) return;
    final active = _currentRescueFeed(pending);
    if (active == null) return;
    final intended = _snapshotFor(active.feed.posts[active.intendedIndex]);
    if (intended?.phase == VideoDeliveryPhase.startable) {
      _clearPendingRescue();
      return;
    }
    _applyRescueDecision(
      active.feed,
      _rescueDecision(active.feed, active.intendedIndex, pending),
    );
  }

  FeedReadyDecision _rescueDecision(
    FeedLoaded current,
    int intendedIndex,
    _PendingTransportRescue pending,
  ) {
    return _readySelector.select(
      current.posts,
      fromIndex: intendedIndex - pending.direction,
      intendedIndex: intendedIndex,
      delivery: _delivery,
      graceExpired: pending.graceExpired,
    );
  }

  void _applyRescueDecision(FeedLoaded current, FeedReadyDecision selected) {
    if (selected.action == FeedReadyAction.rescue) {
      final transition = ++_pageTransition;
      _beginPageTransition(transition);
      _rescueTo(current, selected, transition);
    } else if (selected.action == FeedReadyAction.wait) {
      _ensureRescueTimer();
    }
  }

  void _ensureRescueTimer() {
    _rescueTimer ??= Timer(_readySelector.grace, _expireRescueGrace);
  }

  void _expireRescueGrace() {
    _rescueTimer = null;
    final pending = _awaitingTransportRescue;
    if (pending == null) return;
    _awaitingTransportRescue = (
      deliveryId: pending.deliveryId,
      direction: pending.direction,
      graceExpired: true,
    );
    _rescueAfterDeliveryUpdate();
  }

  void _clearPendingRescue() {
    _awaitingTransportRescue = null;
    _rescueTimer?.cancel();
    _rescueTimer = null;
  }

  VideoDeliverySnapshot? _snapshotFor(VideoPost post) {
    final id = post.media.playbackDeliveryId;
    return id == null ? null : _delivery[id];
  }

  FeedTransportRescue _transportRescue(FeedReadyDecision decision) {
    final reason = switch (decision.reason) {
      FeedReadyReason.etaUnavailable =>
        FeedTransportRescueReason.etaUnavailable,
      FeedReadyReason.etaTooLong => FeedTransportRescueReason.etaTooLong,
      FeedReadyReason.deliveryFailed =>
        FeedTransportRescueReason.deliveryFailed,
      FeedReadyReason.graceExpired => FeedTransportRescueReason.graceExpired,
      _ => throw StateError('Non-rescue decision reached rescue telemetry.'),
    };
    return FeedTransportRescue(
      reason: reason,
      rankDisplacement: decision.displacement,
      wait: _rescueWait(decision.reason == FeedReadyReason.graceExpired),
    );
  }

  Duration _rescueWait(bool expired) =>
      expired ? _readySelector.grace : Duration.zero;
}

final class _RescueCommit {
  const _RescueCommit(this.transition, this.current, this.decision);
  final int transition;
  final FeedLoaded current;
  final FeedReadyDecision decision;
}
