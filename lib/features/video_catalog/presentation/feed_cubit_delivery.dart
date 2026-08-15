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

  void _rescueTo(
    FeedLoaded current,
    FeedReadyDecision decision,
    int transition,
  ) {
    _clearPendingRescue();
    final selected = decision.selectedIndex;
    final moved = _movedTo(current, selected);
    final commit = _RescueCommit(transition, current, moved, decision);
    final preparation = _viewer.prepareToShow(moved.roster.active);
    if (preparation is Future<bool>) {
      return unawaited(_finishRescue(preparation, commit));
    }
    if (!preparation) return;
    _finishRescueNow(commit);
  }

  Future<void> _finishRescue(
    Future<bool> preparation,
    _RescueCommit commit,
  ) async {
    if (!await preparation) return;
    _finishRescueNow(commit);
  }

  void _finishRescueNow(_RescueCommit commit) {
    if (!_acceptsPageTransition(commit.transition, commit.current)) return;
    _pendingTransportJump = commit.moved.activeIndex;
    emit(commit.moved);
    _viewer.rescuedTo(
      commit.moved.posts,
      commit.moved.activeIndex,
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

  void _rememberPendingRescue(
    FeedLoaded current,
    FeedLoaded moved,
    FeedReadyDecision decision,
  ) {
    final intended = decision.intendedIndex;
    final snapshot = _snapshotFor(current.posts[intended]);
    _clearPendingRescue();
    final remapped = moved.activeIndex != intended;
    _awaitingTransportRescue =
        intended == current.activeIndex ||
            snapshot?.phase == VideoDeliveryPhase.startable
        ? null
        : (
            fromIndex: remapped ? -1 : current.activeIndex,
            intendedIndex: moved.activeIndex,
            graceExpired: false,
          );
  }

  void _rescueAfterDeliveryUpdate() {
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
    if (current is! FeedLoaded) return null;
    if (current.activeIndex == pending.intendedIndex) return current;
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
  const _RescueCommit(this.transition, this.current, this.moved, this.decision);

  final int transition;
  final FeedLoaded current;
  final FeedLoaded moved;
  final FeedReadyDecision decision;
}
