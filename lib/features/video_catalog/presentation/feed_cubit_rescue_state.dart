part of 'feed_cubit.dart';

extension FeedCubitRescueState on FeedCubit {
  void _beginLoadGeneration() {
    _pageTransition += 1;
    _cancelPageTransition();
    _clearPendingRescue();
  }

  void _rememberPendingRescue(FeedLoaded current, FeedReadyDecision decision) {
    final intended = decision.intendedIndex;
    _clearPendingRescue();
    final deliveryId = current.posts[intended].media.playbackDeliveryId;
    _awaitingTransportRescue =
        deliveryId == null ||
            intended <= current.activeIndex ||
            decision.reason == FeedReadyReason.intendedReady
        ? null
        : (
            deliveryId: deliveryId,
            direction: intended.compareTo(current.activeIndex),
            graceExpired: false,
          );
  }

  _ActiveTransportRescue? _currentRescueFeed(_PendingTransportRescue pending) {
    final current = state;
    if (current is! FeedLoaded) return null;
    final intended = current.posts.indexWhere(
      (post) => post.media.playbackDeliveryId == pending.deliveryId,
    );
    if (intended >= 0 && current.activeIndex == intended) {
      return (feed: current, intendedIndex: intended);
    }
    _clearPendingRescue();
    return null;
  }

  void _rememberActiveFailure(PlaybackDeliveryId deliveryId) {
    if (_awaitingTransportRescue != null) return;
    final current = state;
    if (current is! FeedLoaded) return;
    final active = current.posts[current.activeIndex].media.playbackDeliveryId;
    if (active != deliveryId) return;
    _awaitingTransportRescue = (
      deliveryId: deliveryId,
      direction: 1,
      graceExpired: false,
    );
  }
}
