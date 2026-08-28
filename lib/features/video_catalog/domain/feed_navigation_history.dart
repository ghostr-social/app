/// How much earlier feed order remains reachable inside one surface.
///
/// This is independent from recommendation replay: watched videos remain
/// excluded from future loads while an explicit backward gesture can revisit
/// the nearby session history.
final class FeedNavigationHistory {
  const FeedNavigationHistory.bounded(int maximumPrevious)
    : assert(maximumPrevious >= 0),
      maximumPrevious = maximumPrevious;

  const FeedNavigationHistory.complete() : maximumPrevious = null;

  /// Matches WARP's three-item planning window behind the current video.
  static const ordinary = FeedNavigationHistory.bounded(3);
  static const unlimited = FeedNavigationHistory.complete();

  final int? maximumPrevious;

  int firstRetained(int activeIndex) {
    RangeError.checkNotNegative(activeIndex, 'activeIndex');
    final maximum = maximumPrevious;
    if (maximum == null || activeIndex <= maximum) return 0;
    return activeIndex - maximum;
  }
}
