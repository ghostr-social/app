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
    final current = state;
    if (current is FeedLoaded) {
      final projected = current.withHlsAuthority(
        snapshot.deliveryId,
        snapshot.hlsAuthority,
      );
      if (!identical(projected, current)) emit(projected);
    }
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
      _readinessEvidence(current),
      fromIndex: current.activeIndex,
      intendedIndex: intended,
    );
  }

  FeedReadinessEvidence _readinessEvidence(FeedLoaded current) {
    return FeedReadinessEvidence(
      posts: current.posts,
      delivery: _delivery,
      preparation: current.preparation,
      verifiedHlsAuthorities: current.verifiedHlsAuthorities,
    );
  }

  void _rescueTo(
    FeedLoaded current,
    FeedReadyDecision decision,
    int transition,
  ) {
    if (_isClosing || isClosed) return _completePageTransition(transition);
    _clearPendingRescue();
    final commit = _RescueCommit(transition, current, decision);
    _finishRescueNow(commit);
  }

  void _finishRescueNow(_RescueCommit commit) {
    final current = _acceptedPageTransition(commit.transition, commit.current);
    if (current == null) return _completePageTransition(commit.transition);
    final moved = _presentedAt(current, commit.decision.selectedIndex);
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
      _readinessEvidence(current),
      fromIndex: intendedIndex - pending.direction,
      intendedIndex: intendedIndex,
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
    } else if (selected.reason == FeedReadyReason.intendedReady) {
      _settlePendingRescue();
    }
  }

  void _settlePendingRescue() {
    final pending = _awaitingTransportRescue;
    if (pending == null) return;
    _awaitingTransportRescue = (
      deliveryId: pending.deliveryId,
      direction: pending.direction,
      graceExpired: false,
    );
    _rescueTimer?.cancel();
    _rescueTimer = null;
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
