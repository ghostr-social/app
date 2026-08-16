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
    _rescueAfterDeliveryUpdate();
  }

  bool _consumeTransportJump(int index) {
    final pending = _pendingTransportJump;
    _pendingTransportJump = null;
    final current = state;
    return pending == index &&
        current is FeedLoaded &&
        current.activeIndex == index;
  }

  FeedReadyDecision _readyDecision(FeedLoaded current, int intended) {
    return _readySelector.select(
      current.posts,
      fromIndex: current.activeIndex,
      intendedIndex: intended,
      delivery: intended < current.activeIndex ? const {} : _delivery,
    );
  }

  bool _rescueTo(
    FeedLoaded current,
    FeedReadyDecision decision,
    int transition,
  ) {
    _clearPendingRescue();
    final selected = decision.selectedIndex;
    final moved = _movedTo(current, selected);
    final commit = (
      transition: transition,
      current: current,
      index: selected,
      target: VideoInteractionTarget.fromPost(moved.roster.active),
      decision: decision,
    );
    return _finishRescueNow(commit);
  }

  bool _finishRescueNow(_NavigationCommit commit) {
    final current = _acceptedNavigation(commit);
    if (current == null) return false;
    final moved = _movedTo(current, commit.index);
    _pendingTransportJump = moved.activeIndex;
    emit(moved);
    _viewer.rescuedTo(
      moved.posts,
      moved.activeIndex,
      _transportRescue(commit.decision),
    );
    _ensureBuffered();
    return true;
  }

  Future<void> _stopDeliveryUpdates() async {
    _clearPendingRescue();
    final subscription = _deliverySubscription;
    _deliverySubscription = null;
    await subscription?.cancel();
  }

  void _rememberPendingRescue(
    FeedLoaded current,
    FeedLoaded moved,
    FeedReadyDecision decision,
  ) {
    final intended = decision.intendedIndex;
    final snapshot = _snapshotFor(current.posts[intended]);
    _clearPendingRescue();
    _awaitingTransportRescue =
        intended <= current.activeIndex ||
            snapshot?.phase == VideoDeliveryPhase.startable
        ? null
        : (
            fromIndex: current.activeIndex,
            intendedIndex: moved.activeIndex,
            intendedTarget: VideoInteractionTarget.fromPost(
              moved.roster.active,
            ),
            graceExpired: false,
          );
  }

  void _rescueAfterDeliveryUpdate() {
    if (!_isSurfaceVisible) return;
    final pending = _awaitingTransportRescue;
    if (pending == null) return;
    final current = _currentRescueFeed(pending);
    if (current == null) return;
    final intended = _snapshotFor(current.posts[pending.intendedIndex]);
    if (intended?.phase == VideoDeliveryPhase.startable) {
      _clearPendingRescue();
      return;
    }
    _applyRescueDecision(current, _rescueDecision(current, pending));
  }

  FeedLoaded? _currentRescueFeed(_PendingTransportRescue pending) {
    final current = state;
    if (current is FeedLoaded &&
        current.activeIndex == pending.intendedIndex &&
        VideoInteractionTarget.fromPost(current.roster.active) ==
            pending.intendedTarget) {
      return current;
    }
    _clearPendingRescue();
    return null;
  }

  FeedReadyDecision _rescueDecision(
    FeedLoaded current,
    _PendingTransportRescue pending,
  ) {
    return _readySelector.select(
      current.posts,
      fromIndex: pending.fromIndex,
      intendedIndex: pending.intendedIndex,
      delivery: _delivery,
      graceExpired: pending.graceExpired,
    );
  }

  void _applyRescueDecision(FeedLoaded current, FeedReadyDecision selected) {
    if (selected.action == FeedReadyAction.rescue) {
      final transition = ++_pageTransition;
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
      fromIndex: pending.fromIndex,
      intendedIndex: pending.intendedIndex,
      intendedTarget: pending.intendedTarget,
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
      wait: decision.reason == FeedReadyReason.graceExpired
          ? _readySelector.grace
          : Duration.zero,
    );
  }
}
