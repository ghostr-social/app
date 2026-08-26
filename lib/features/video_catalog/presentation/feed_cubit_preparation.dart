part of 'feed_cubit.dart';

extension FeedCubitPreparation on FeedCubit {
  void _startPreparationUpdates() {
    final updates = _dependencies.preparationUpdates;
    if (updates == null) return;
    _preparationAvailable = true;
    try {
      _listenPreparation(updates);
    } on Object catch (error, stackTrace) {
      _preparationFailed(error, stackTrace);
    }
  }

  void _listenPreparation(PlaybackPreparationUpdates updates) {
    var terminal = false;
    final subscription = updates.watchPreparation().listen(
      _acceptPreparation,
      onError: (Object error, StackTrace stackTrace) {
        terminal = true;
        _preparationFailed(error, stackTrace);
      },
      onDone: () {
        terminal = true;
        _disablePreparation();
      },
      cancelOnError: true,
    );
    if (terminal) return unawaited(subscription.cancel());
    _preparationSubscription = subscription;
  }

  void _preparationFailed(Object error, StackTrace stackTrace) {
    _disablePreparation();
    _reportUpdateError(error, stackTrace);
  }

  void _disablePreparation() {
    _preparationAvailable = false;
    _preparationSubscription = null;
    final current = state;
    if (current is FeedLoaded) {
      emit(current.withPreparation(FeedPlaybackPreparation.unmanaged()));
    }
  }

  void _acceptPreparation(PlaybackPreparationPlan plan) {
    _preparationAvailable = true;
    final current = state;
    if (current is! FeedLoaded) {
      _preparation.observe(plan);
      return;
    }
    final accepted = _preparation.acceptWindow(
      plan,
      current.roster.active.media,
      _upcomingMedia(current),
    );
    if (accepted == null) return;
    emit(current.withPreparation(accepted));
    _rescueAfterDeliveryUpdate();
  }

  FeedLoaded _projectPreparation(FeedLoaded feed) {
    if (!_preparationAvailable) return feed;
    return feed.withPreparation(
      _preparation.projectWindow(
        feed.roster.active.media,
        _upcomingMedia(feed),
      ),
    );
  }

  FeedLoaded _realignPreparation(FeedLoaded previous, FeedLoaded moved) {
    if (!_preparationAvailable) return moved;
    final preparation = _preparation.realignWindow(
      previous.preparation,
      moved.roster.active.media,
      _upcomingMedia(moved),
    );
    return moved.withPreparation(preparation);
  }

  List<VideoMediaSource> _upcomingMedia(FeedLoaded feed) {
    return feed.posts
        .skip(feed.activeIndex + 1)
        .map((post) => post.media)
        .toList(growable: false);
  }

  Future<void> _stopPreparationUpdates() async {
    final subscription = _preparationSubscription;
    _preparationSubscription = null;
    await subscription?.cancel();
  }
}
