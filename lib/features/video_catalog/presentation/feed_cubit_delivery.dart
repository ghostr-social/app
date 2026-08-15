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

  int _selectedPage(FeedLoaded current, int intended) {
    return _readySelector.select(
      current.posts,
      fromIndex: current.activeIndex,
      intendedIndex: intended,
      delivery: _delivery,
    );
  }

  void _rescueTo(FeedLoaded current, int selected) {
    _awaitingTransportRescue = null;
    _pendingTransportJump = selected;
    emit(current.withPage(selected));
    _viewer.rescuedTo(current.posts, selected);
    _ensureBuffered();
  }

  Future<void> _stopDeliveryUpdates() async {
    _awaitingTransportRescue = null;
    final subscription = _deliverySubscription;
    _deliverySubscription = null;
    await subscription?.cancel();
  }

  void _rememberPendingRescue(FeedLoaded current, int intended) {
    final snapshot = _snapshotFor(current.posts[intended]);
    _awaitingTransportRescue = intended == current.activeIndex ||
            snapshot?.phase == VideoDeliveryPhase.startable
        ? null
        : (fromIndex: current.activeIndex, intendedIndex: intended);
  }

  void _rescueAfterDeliveryUpdate() {
    final pending = _awaitingTransportRescue;
    final current = state;
    if (pending == null || current is! FeedLoaded) return;
    if (current.activeIndex != pending.intendedIndex) {
      _awaitingTransportRescue = null;
      return;
    }
    final intended = _snapshotFor(current.posts[pending.intendedIndex]);
    if (intended?.phase == VideoDeliveryPhase.startable) {
      _awaitingTransportRescue = null;
      return;
    }
    final selected = _readySelector.select(
      current.posts,
      fromIndex: pending.fromIndex,
      intendedIndex: pending.intendedIndex,
      delivery: _delivery,
    );
    if (selected != pending.intendedIndex) _rescueTo(current, selected);
  }

  VideoDeliverySnapshot? _snapshotFor(VideoPost post) {
    final id = post.media.playbackDeliveryId;
    return id == null ? null : _delivery[id];
  }
}
