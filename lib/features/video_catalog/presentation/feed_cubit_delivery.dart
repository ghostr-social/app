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

  void _rescueTo(FeedLoaded current, FeedReadyDecision decision) {
    _clearPendingRescue();
    final selected = decision.selectedIndex;
    _pendingTransportJump = selected;
    emit(current.withPage(selected));
    _viewer.rescuedTo(current.posts, selected, _transportRescue(decision));
    _ensureBuffered();
  }

  Future<void> _stopDeliveryUpdates() async {
    _clearPendingRescue();
    final subscription = _deliverySubscription;
    _deliverySubscription = null;
    await subscription?.cancel();
  }

  void _rememberPendingRescue(FeedLoaded current, FeedReadyDecision decision) {
    final intended = decision.intendedIndex;
    final snapshot = _snapshotFor(current.posts[intended]);
    _clearPendingRescue();
    _awaitingTransportRescue =
        intended == current.activeIndex ||
            snapshot?.phase == VideoDeliveryPhase.startable
        ? null
        : (
            fromIndex: current.activeIndex,
            intendedIndex: intended,
            graceExpired: false,
          );
    if (decision.action == FeedReadyAction.wait) _ensureRescueTimer();
  }

  void _rescueAfterDeliveryUpdate() {
    final pending = _awaitingTransportRescue;
    final current = state;
    if (pending == null || current is! FeedLoaded) return;
    if (current.activeIndex != pending.intendedIndex) {
      _clearPendingRescue();
      return;
    }
    final intended = _snapshotFor(current.posts[pending.intendedIndex]);
    if (intended?.phase == VideoDeliveryPhase.startable) {
      _clearPendingRescue();
      return;
    }
    final selected = _readySelector.select(
      current.posts,
      fromIndex: pending.fromIndex,
      intendedIndex: pending.intendedIndex,
      delivery: _delivery,
      graceExpired: pending.graceExpired,
    );
    if (selected.action == FeedReadyAction.rescue) {
      _rescueTo(current, selected);
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
