part of 'feed_cubit.dart';

final class _FeedUpdateState {
  _FeedUpdateState(this.retry);

  final FeedUpdateRetry retry;
  StreamSubscription<VideoFeedUpdate>? subscription;
  FeedKind? kind;
  int feed = 0;
  int listener = 0;
  int pulls = 0;
  BigInt revision = BigInt.from(-1);
  BigInt? pendingRevision;
  int? reloadingFeed;
}

extension FeedCubitUpdates on FeedCubit {
  Future<void> _runFeedPull(Future<void> Function() pull) async {
    _updates.pulls += 1;
    try {
      await pull();
    } finally {
      _updates.pulls -= 1;
      _startPendingFeedUpdate();
    }
  }

  Future<void> _ensureFeedUpdates(FeedKind kind) async {
    if (_dependencies.updates == null) return;
    if (_updates.kind == kind && _updates.subscription != null) return;
    await _replaceFeedUpdates(kind);
  }

  Future<void> _refreshFeedUpdates(FeedKind kind) async {
    final updates = _dependencies.updates;
    if (kind == FeedKind.following && updates != null) {
      if (updates case VideoFeedUpdateRefreshPolicy policy) {
        if (!await policy.shouldRebind(kind)) return;
      }
      return _replaceFeedUpdates(kind);
    }
    await _ensureFeedUpdates(kind);
  }

  Future<void> _replaceFeedUpdates(FeedKind kind) async {
    final listener = ++_updates.listener;
    if (_updates.kind == kind) {
      _updates.retry.cancel();
      _updates.revision = BigInt.from(-1);
    } else {
      _updates.retry.reset();
      _prepareFeedUpdates(kind);
    }
    await _cancelFeedUpdateSubscription();
    if (_cannotStartFeedUpdates(listener)) return;
    _listenForFeedUpdates(listener, kind);
  }

  bool _cannotStartFeedUpdates(int listener) {
    return _dependencies.updates == null ||
        isClosed ||
        listener != _updates.listener;
  }

  void _prepareFeedUpdates(FeedKind kind) {
    _updates.feed += 1;
    _updates.kind = kind;
    _updates.revision = BigInt.from(-1);
    _updates.pendingRevision = null;
  }

  void _listenForFeedUpdates(int listener, FeedKind kind) {
    try {
      _updates.subscription = _dependencies.updates!
          .watchFeed(kind)
          .listen(
            (update) => _acceptFeedUpdate(listener, kind, update),
            onError: (Object error, StackTrace stackTrace) =>
                _feedUpdatesFailed(listener, kind, error, stackTrace),
            onDone: () => _feedUpdatesEnded(listener, kind),
            cancelOnError: true,
          );
    } on Object catch (error, stackTrace) {
      _feedUpdatesFailed(listener, kind, error, stackTrace);
    }
  }

  void _acceptFeedUpdate(int listener, FeedKind kind, VideoFeedUpdate update) {
    if (!_acceptsFeedUpdate(listener, kind)) return;
    if (update.revision <= _updates.revision) return;
    _updates.revision = update.revision;
    _updates.retry.succeeded();
    if (!update.hasPosts) return;
    _updates.pendingRevision = update.revision;
    _startPendingFeedUpdate();
  }

  bool _acceptsFeedUpdate(int listener, FeedKind kind) {
    return listener == _updates.listener &&
        _acceptsFeedReconciliation(_updates.feed, kind);
  }

  bool _acceptsFeedReconciliation(int feed, FeedKind kind) {
    return !isClosed &&
        feed == _updates.feed &&
        _updates.kind == kind &&
        state.kind == kind;
  }

  void _startPendingFeedUpdate() {
    final kind = _updates.kind;
    if (kind == null || !_canDrainFeedUpdates(kind)) return;
    final feed = _updates.feed;
    _updates.reloadingFeed = feed;
    unawaited(_drainFeedUpdates(feed, kind));
  }

  bool _canDrainFeedUpdates(FeedKind kind) {
    return _updates.reloadingFeed != _updates.feed &&
        _updates.pulls == 0 &&
        _updates.pendingRevision != null &&
        state is! FeedLoading &&
        _acceptsFeedReconciliation(_updates.feed, kind);
  }

  Future<void> _drainFeedUpdates(int feed, FeedKind kind) async {
    try {
      while (_hasPendingFeedUpdate(feed, kind)) {
        _updates.pendingRevision = null;
        await _reloadFromFeedUpdate(feed, kind);
      }
    } finally {
      if (_updates.reloadingFeed == feed) _updates.reloadingFeed = null;
      _startPendingFeedUpdate();
    }
  }

  bool _hasPendingFeedUpdate(int feed, FeedKind kind) {
    return _updates.pulls == 0 &&
        _updates.pendingRevision != null &&
        state is! FeedLoading &&
        _acceptsFeedReconciliation(feed, kind);
  }

  Future<void> _reloadFromFeedUpdate(int feed, FeedKind kind) async {
    final loaded = state is FeedLoaded;
    final result = await _loads.newest(
      () => loaded ? _fetch.resync(kind) : _fetch.unwatched(kind),
    );
    if (!_acceptsFeedReconciliation(feed, kind) || result == null) return;
    if (result case FeedFetched(
      :final posts,
      :final eligiblePosts,
    ) when posts.isNotEmpty) {
      final current = state;
      current is FeedLoaded
          ? _acceptRefresh(current, posts, eligiblePosts)
          : _acceptLoad(kind, posts);
    }
  }

  void _feedUpdatesEnded(int listener, FeedKind kind) {
    if (!_acceptsFeedUpdate(listener, kind)) return;
    _updates.subscription = null;
    _updates.retry.schedule(() => unawaited(_replaceFeedUpdates(kind)));
  }

  void _feedUpdatesFailed(
    int listener,
    FeedKind kind,
    Object error,
    StackTrace stackTrace,
  ) {
    if (!_acceptsFeedUpdate(listener, kind)) return;
    _reportUpdateError(error, stackTrace);
    _feedUpdatesEnded(listener, kind);
  }

  Future<void> _cancelFeedUpdateSubscription() async {
    final subscription = _updates.subscription;
    _updates.subscription = null;
    await subscription?.cancel();
  }

  Future<void> _stopFeedUpdates() async {
    _updates.feed += 1;
    _updates.listener += 1;
    _updates.retry.reset();
    _updates.kind = null;
    _updates.pendingRevision = null;
    await _cancelFeedUpdateSubscription();
  }

  void _acknowledgePendingFeedUpdate() {
    _updates.pendingRevision = null;
  }
}
