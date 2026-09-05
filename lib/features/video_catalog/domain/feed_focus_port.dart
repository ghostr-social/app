import 'package:ghostr/features/video_catalog/domain/video_post.dart';

enum FeedFocusCause { userNavigation, rosterChange, transportRescue }

enum FeedTransportRescueReason {
  etaUnavailable,
  etaTooLong,
  deliveryFailed,
  graceExpired,
}

final class FeedTransportRescue {
  const FeedTransportRescue({
    required this.reason,
    required this.rankDisplacement,
    required this.wait,
  });

  final FeedTransportRescueReason reason;
  final int rankDisplacement;
  final Duration wait;
}

/// The viewer's position in a feed as the delivery engine sees it: an
/// ordered window of posts around — and including — the current item.
final class FeedFocus {
  FeedFocus._(
    this.window,
    this.currentIndex,
    this.watched,
    this.cause,
    this.rescue,
  );

  /// Preserves the complete feed order so the delivery policy can derive
  /// its own adaptive frontier around [activeIndex].
  factory FeedFocus.around({
    required List<VideoPost> posts,
    required int activeIndex,
    Duration watched = Duration.zero,
    FeedFocusCause cause = FeedFocusCause.userNavigation,
    FeedTransportRescue? rescue,
  }) {
    RangeError.checkValidIndex(activeIndex, posts, 'activeIndex');
    if ((cause == FeedFocusCause.transportRescue) != (rescue != null)) {
      throw ArgumentError('Transport rescue focus requires rescue context.');
    }
    return FeedFocus._(
      List<VideoPost>.unmodifiable(posts),
      activeIndex,
      watched,
      cause,
      rescue,
    );
  }

  /// Complete feed-ordered roster; always contains [current].
  final List<VideoPost> window;

  /// Index of the current post within [window].
  final int currentIndex;

  /// Time spent watching the current post. Dart has no playback watch
  /// timer yet, so callers pass zero until playback stats exist.
  final Duration watched;

  /// Why the visible position changed; transport rescue is not engagement.
  final FeedFocusCause cause;

  /// Present only when delivery substituted a bounded semantic neighbor.
  final FeedTransportRescue? rescue;

  VideoPost get current => window[currentIndex];
}

/// Domain port announcing where the viewer is, so the media delivery
/// engine can reprioritize downloads around the focus window.
abstract interface class FeedFocusPort {
  /// Replaces the engine's focus window. Implementations must never
  /// throw into the caller — delivery trouble is theirs to absorb.
  void focusChanged(FeedFocus focus);
}

/// Global delivery ownership, including the absence of a visible feed.
abstract interface class FeedFocusSink implements FeedFocusPort {
  void clearFocus();
}

/// Exclusive ownership of the shared delivery focus for one feed surface.
abstract interface class FeedFocusLease implements FeedFocusPort {
  void activate();

  void deactivate();

  void release();
}
