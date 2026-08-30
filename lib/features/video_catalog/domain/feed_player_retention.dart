import 'feed_navigation_history.dart';

/// Balances recent-player reuse against the shared playback-controller budget.
final class FeedPlayerRetention {
  const FeedPlayerRetention({
    required this.maximumControllers,
    required this.minimumPrevious,
    required this.history,
  }) : assert(maximumControllers > 0),
       assert(minimumPrevious >= 0),
       assert(minimumPrevious < maximumControllers);

  final int maximumControllers;
  final int minimumPrevious;
  final FeedNavigationHistory history;

  int warmPreviousDepth({
    required int preparedFutureCount,
    bool canReplenish = false,
  }) {
    RangeError.checkNotNegative(preparedFutureCount, 'preparedFutureCount');
    final futureCapacity = maximumControllers - minimumPrevious - 1;
    final headroom = canReplenish && preparedFutureCount < futureCapacity
        ? 1
        : 0;
    final available = maximumControllers - preparedFutureCount - headroom - 1;
    if (available <= 0) {
      return 0;
    }
    final maximumPrevious = history.maximumPrevious;
    if (maximumPrevious == null || available < maximumPrevious) {
      return available;
    }
    return maximumPrevious;
  }
}
